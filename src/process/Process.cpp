#include "Process.hpp"

#include <cassert>
#include <cerrno>
#include <cstddef>
#include <cstdlib>
#include <sstream>
#include <stdexcept>
#include <utility>

#include <fcntl.h>
#include <unistd.h>
#include <sys/wait.h>

#include "bdrck/cwrap/String.hpp"
#include "bdrck/util/Error.hpp"

namespace
{
constexpr std::size_t READ_BUFFER_SIZE = 256;

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
	argv.emplace_back(bdrck::cwrap::string::strdup(path.c_str()),
	                  argvDeleter);
	for(auto const &argument : arguments)
	{
		argv.emplace_back(
		        bdrck::cwrap::string::strdup(argument.c_str()),
		        argvDeleter);
	}
	return argv;
}

std::vector<char *>
toArgvPointers(bdrck::process::ProcessArguments::ArgvContainer const &argv)
{
	std::vector<char *> pointers(argv.size(), nullptr);
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

void renamePipe(int srcFd, int dstFd)
{
	int ret = dup2(srcFd, dstFd);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
	closePipe(srcFd);
}

[[noreturn]] void throwChildSignalError(int sig)
{
	throw std::runtime_error(bdrck::cwrap::string::strsignal(sig));
}
}

namespace bdrck
{
namespace process
{
namespace detail
{
Pipe::Pipe(int flags) : read(-1), write(-1)
{
	int pipefd[2];
	int ret = pipe2(pipefd, flags);
	if(ret == -1)
		util::error::throwErrnoError();
	read = pipefd[0];
	write = pipefd[1];
}
}

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
        : args(p, a), parent(getpid()), child(-1), pipes()
{
	// Open a pipe, so we can get error messages from our child.

	detail::Pipe errorPipe;
	addPipeFlags(errorPipe.write, O_CLOEXEC);

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

		closePipe(errorPipe.read);

		closePipe(pipes[terminal::StdStream::In].write);
		closePipe(pipes[terminal::StdStream::Out].read);
		closePipe(pipes[terminal::StdStream::Err].read);

		renamePipe(pipes[terminal::StdStream::In].read,
		           terminal::streamFD(terminal::StdStream::In));
		renamePipe(pipes[terminal::StdStream::Out].write,
		           terminal::streamFD(terminal::StdStream::Out));
		renamePipe(pipes[terminal::StdStream::Err].write,
		           terminal::streamFD(terminal::StdStream::Err));

		try
		{
			// The POSIX standard guarantees that argv will not be
			// modified, so this const cast is safe.
			if(execvp(args.file, args.argv) == -1)
				util::error::throwErrnoError();
		}
		catch(std::runtime_error const &e)
		{
			std::string message = e.what();
			ssize_t written =
			        write(errorPipe.write, message.c_str(),
			              message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		catch(...)
		{
			std::string message = "Unknown error.";
			ssize_t written =
			        write(errorPipe.write, message.c_str(),
			              message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		_exit(EXIT_FAILURE);
	}
	else
	{
		// Still in the parent process. Check for errors.

		child = pid;

		closePipe(errorPipe.write);

		closePipe(pipes[terminal::StdStream::In].read);
		closePipe(pipes[terminal::StdStream::Out].write);
		closePipe(pipes[terminal::StdStream::Err].write);

		std::string error = readAll(errorPipe.read);
		closePipe(errorPipe.read);
		if(!error.empty())
			throw std::runtime_error(error);
	}
}

Process::~Process()
{
	try
	{
		closePipe(pipes[terminal::StdStream::In].write);
		closePipe(pipes[terminal::StdStream::Out].read);
		closePipe(pipes[terminal::StdStream::Err].read);

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
		return pipes.at(stream).write;

	case terminal::StdStream::Out:
	case terminal::StdStream::Err:
		return pipes.at(stream).read;
	}
	return -1;
}

int Process::wait()
{
	if(child == -1)
		return EXIT_SUCCESS;

	int status;
	while(waitpid(child, &status, 0) == -1)
	{
		if(errno != EINTR)
			util::error::throwErrnoError();
	}

	child = -1;
	if(WIFEXITED(status))
		return WEXITSTATUS(status);
	else if(WIFSIGNALED(status))
		throwChildSignalError(WTERMSIG(status));

	return EXIT_FAILURE;
}
}
}
