#include "Process.hpp"

#include <cassert>
#include <cerrno>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <stdexcept>
#include <utility>

#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>

#include "bdrck/util/Error.hpp"

namespace
{
constexpr int INVALID_PIPE_VALUE = -1;

enum class PipeSide
{
	READ,
	WRITE
};
}

namespace bdrck
{
namespace process
{
namespace detail
{
class Pipe
{
public:
	explicit Pipe(int flags = 0);

	Pipe(Pipe const &) = default;
	Pipe(Pipe &&) = default;
	Pipe &operator=(Pipe const &) = default;
	Pipe &operator=(Pipe &&) = default;

	~Pipe() = default;

	int getSide(PipeSide side) const;

private:
	int read;
	int write;
};

Pipe::Pipe(int flags) : read(INVALID_PIPE_VALUE), write(INVALID_PIPE_VALUE)
{
	int pipefd[2];
	int ret = pipe2(pipefd, flags);
	if(ret == -1)
		util::error::throwErrnoError();
	read = pipefd[0];
	write = pipefd[1];
}

int Pipe::getSide(PipeSide side) const
{
	switch(side)
	{
	case PipeSide::READ:
		return read;
	case PipeSide::WRITE:
		return write;
	}
}

struct Pid
{
	pid_t pid;

	Pid(pid_t p);

	Pid(Pid const &) = default;
	Pid(Pid &&) = default;
	Pid &operator=(Pid const &) = default;
	Pid &operator=(Pid &&) = default;

	~Pid() = default;

	int compare(Pid const &o) const;
	bool operator==(Pid const &o) const;
	bool operator!=(Pid const &o) const;
	bool operator<(Pid const &o) const;
	bool operator<=(Pid const &o) const;
	bool operator>(Pid const &o) const;
	bool operator>=(Pid const &o) const;
};

Pid::Pid(pid_t p) : pid(p)
{
}

int Pid::compare(Pid const &o) const
{
	if(pid < o.pid)
		return -1;
	else if(pid > o.pid)
		return 1;
	else
		return 0;
}

bool Pid::operator==(Pid const &o) const
{
	return compare(o) == 0;
}

bool Pid::operator!=(Pid const &o) const
{
	return compare(o) != 0;
}

bool Pid::operator<(Pid const &o) const
{
	return compare(o) < 0;
}

bool Pid::operator<=(Pid const &o) const
{
	return compare(o) <= 0;
}

bool Pid::operator>(Pid const &o) const
{
	return compare(o) > 0;
}

bool Pid::operator>=(Pid const &o) const
{
	return compare(o) >= 0;
}
}
}
}

namespace
{
constexpr std::size_t READ_BUFFER_SIZE = 256;

char *safeStrdup(char const *s)
{
	char *copy = ::strdup(s);
	if(copy == nullptr)
		::bdrck::util::error::throwErrnoError();
	return copy;
}

void argvDeleter(char *p)
{
	std::free(p);
}

bdrck::process::ProcessArguments::ArgvContainer
duplicateArgvStrings(std::string const &path,
                     std::vector<std::string> const &arguments)
{
	bdrck::process::ProcessArguments::ArgvContainer argv;
	argv.reserve(arguments.size() + 1);
	argv.emplace_back(safeStrdup(path.c_str()), argvDeleter);
	for(auto const &argument : arguments)
	{
		argv.emplace_back(safeStrdup(argument.c_str()), argvDeleter);
	}
	return argv;
}

std::vector<char *>
toArgvPointers(bdrck::process::ProcessArguments::ArgvContainer const &argv)
{
	std::vector<char *> pointers(argv.size() + 1, nullptr);
	for(std::size_t i = 0; i < argv.size(); ++i)
		pointers[i] = argv[i].get();
	return pointers;
}

void addPipeFlags(int fd, int flags)
{
	int existingFlags = fcntl(fd, F_GETFD);
	if(existingFlags == -1)
		bdrck::util::error::throwErrnoError();

	if(fcntl(fd, F_SETFD, existingFlags | flags) == -1)
		bdrck::util::error::throwErrnoError();
}

std::string readAll(int fd)
{
	std::vector<char> buffer(READ_BUFFER_SIZE);
	std::ostringstream oss;
	ssize_t count;
	while((count = read(fd, buffer.data(), buffer.size())) != 0)
	{
		if(count == -1)
			bdrck::util::error::throwErrnoError();
		oss << std::string(buffer.data(),
		                   static_cast<std::size_t>(count));
	}
	return oss.str();
}

void closePipe(int fd)
{
	int ret = close(fd);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
}

void closeParentSide(std::map<bdrck::process::terminal::StdStream,
                              bdrck::process::detail::Pipe> const &pipes)
{
	closePipe(pipes.at(bdrck::process::terminal::StdStream::In)
	                  .getSide(PipeSide::WRITE));
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Out)
	                  .getSide(PipeSide::READ));
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Err)
	                  .getSide(PipeSide::READ));
}

void closeChildSide(std::map<bdrck::process::terminal::StdStream,
                             bdrck::process::detail::Pipe> const &pipes)
{
	closePipe(pipes.at(bdrck::process::terminal::StdStream::In)
	                  .getSide(PipeSide::READ));
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Out)
	                  .getSide(PipeSide::WRITE));
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Err)
	                  .getSide(PipeSide::WRITE));
}

void renamePipe(int srcFd, int dstFd)
{
	int ret = dup2(srcFd, dstFd);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
	closePipe(srcFd);
}

[[noreturn]] void throwChildSignalError(int sig)
{
	char *message = ::strsignal(sig);
	if(message != nullptr)
		throw std::runtime_error(message);
	else
		throw std::runtime_error("Unrecognized signal.");
}

int waitOnPid(bdrck::process::detail::Pid &pid)
{
	if(pid.pid == -1)
		return EXIT_SUCCESS;

	int status;
	while(waitpid(pid.pid, &status, 0) == -1)
	{
		if(errno != EINTR)
			bdrck::util::error::throwErrnoError();
	}

	pid.pid = -1;
	if(WIFEXITED(status))
		return WEXITSTATUS(status);
	else if(WIFSIGNALED(status))
		throwChildSignalError(WTERMSIG(status));

	return EXIT_FAILURE;
}
}

namespace bdrck
{
namespace process
{
ProcessArguments::ProcessArguments(std::string const &p,
                                   std::vector<std::string> const &a)
        : path(p),
          arguments(a),
          argvContainer(duplicateArgvStrings(path, arguments)),
          argvPointers(toArgvPointers(argvContainer)),
          file(path.c_str()),
          argv(argvPointers.data())
{
}

Process::Process(std::string const &p, std::vector<std::string> const &a)
        : args(p, a),
          parent(std::make_unique<detail::Pid>(getpid())),
          child(std::make_unique<detail::Pid>(-1)),
          pipes()
{
	// Open a pipe, so we can get error messages from our child.

	detail::Pipe errorPipe;
	addPipeFlags(errorPipe.getSide(PipeSide::WRITE), O_CLOEXEC);

	// Open pipes for the child's standard streams.
	pipes.emplace(std::make_pair<terminal::StdStream, detail::Pipe>(
	        terminal::StdStream::In, detail::Pipe()));
	pipes.emplace(std::make_pair<terminal::StdStream, detail::Pipe>(
	        terminal::StdStream::Out, detail::Pipe()));
	pipes.emplace(std::make_pair<terminal::StdStream, detail::Pipe>(
	        terminal::StdStream::Err, detail::Pipe()));

	// Fork a new process.

	pid_t pid = fork();
	if(pid == -1)
		util::error::throwErrnoError();

	if(pid == 0)
	{
		// In the child process. Try to exec the binary.

		try
		{
			closePipe(errorPipe.getSide(PipeSide::READ));

			closeParentSide(pipes);

			renamePipe(pipes[terminal::StdStream::In].getSide(
			                   PipeSide::READ),
			           terminal::streamFileno(
			                   terminal::StdStream::In));
			renamePipe(pipes[terminal::StdStream::Out].getSide(
			                   PipeSide::WRITE),
			           terminal::streamFileno(
			                   terminal::StdStream::Out));
			renamePipe(pipes[terminal::StdStream::Err].getSide(
			                   PipeSide::WRITE),
			           terminal::streamFileno(
			                   terminal::StdStream::Err));

			// The POSIX standard guarantees that argv will not be
			// modified, so this const cast is safe.
			if(execvp(args.file, args.argv) == -1)
				util::error::throwErrnoError();
		}
		catch(std::runtime_error const &e)
		{
			std::string message = e.what();
			ssize_t written =
			        write(errorPipe.getSide(PipeSide::WRITE),
			              message.c_str(), message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		catch(...)
		{
			std::string message = "Unknown error.";
			ssize_t written =
			        write(errorPipe.getSide(PipeSide::WRITE),
			              message.c_str(), message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		_exit(EXIT_FAILURE);
	}
	else
	{
		// Still in the parent process. Check for errors.

		child->pid = pid;

		closePipe(errorPipe.getSide(PipeSide::WRITE));

		closeChildSide(pipes);

		std::string error = readAll(errorPipe.getSide(PipeSide::READ));
		closePipe(errorPipe.getSide(PipeSide::READ));
		if(!error.empty())
			throw std::runtime_error(error);
	}
}

Process::~Process()
{
	try
	{
		closeParentSide(pipes);
		wait();
	}
	catch(...)
	{
	}
}

int Process::getPipe(terminal::StdStream stream) const
{
	switch(stream)
	{
	case terminal::StdStream::In:
		return pipes.at(stream).getSide(PipeSide::WRITE);

	case terminal::StdStream::Out:
	case terminal::StdStream::Err:
		return pipes.at(stream).getSide(PipeSide::READ);
	}
	return INVALID_PIPE_VALUE;
}

int Process::wait()
{
	return waitOnPid(*child);
}
}
}
