#include "Process.hpp"

#include <cassert>
#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <stdexcept>
#include <utility>

#include "bdrck/process/PipeCast.hpp"
#include "bdrck/util/Error.hpp"

#ifdef _WIN32
#include <Windows.h>
#else
#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#endif

namespace
{
#ifdef _WIN32
typedef HANDLE NativeProcessHandle;
constexpr NativeProcessHandle INVALID_PROCESS_HANDLE_VALUE =
        INVALID_HANDLE_VALUE;
#else
typedef pid_t NativeProcessHandle;
constexpr NativeProcessHandle INVALID_PROCESS_HANDLE_VALUE = -1;
#endif

NativeProcessHandle getCurrentProcessHandle()
{
#ifdef _WIN32
	return GetCurrentProcess();
#else
	return getpid();
#endif
}
}

namespace bdrck
{
namespace process
{
namespace detail
{
struct ProcessHandle
{
	NativeProcessHandle handle;

	ProcessHandle(NativeProcessHandle h);

	ProcessHandle(ProcessHandle const &) = default;
	ProcessHandle(ProcessHandle &&) = default;
	ProcessHandle &operator=(ProcessHandle const &) = default;
	ProcessHandle &operator=(ProcessHandle &&) = default;

	~ProcessHandle() = default;

	int compare(ProcessHandle const &o) const;
	bool operator==(ProcessHandle const &o) const;
	bool operator!=(ProcessHandle const &o) const;
	bool operator<(ProcessHandle const &o) const;
	bool operator<=(ProcessHandle const &o) const;
	bool operator>(ProcessHandle const &o) const;
	bool operator>=(ProcessHandle const &o) const;
};

ProcessHandle::ProcessHandle(NativeProcessHandle h) : handle(h)
{
}

int ProcessHandle::compare(ProcessHandle const &o) const
{
#ifdef _WIN32
	DWORD comparable = GetProcessId(handle);
	DWORD oComparable = GetProcessId(o.handle);
#else
	auto comparable = handle;
	auto oComparable = o.handle;
#endif

	if(comparable < oComparable)
		return -1;
	else if(comparable > oComparable)
		return 1;
	else
		return 0;
}

bool ProcessHandle::operator==(ProcessHandle const &o) const
{
	return compare(o) == 0;
}

bool ProcessHandle::operator!=(ProcessHandle const &o) const
{
	return compare(o) != 0;
}

bool ProcessHandle::operator<(ProcessHandle const &o) const
{
	return compare(o) < 0;
}

bool ProcessHandle::operator<=(ProcessHandle const &o) const
{
	return compare(o) <= 0;
}

bool ProcessHandle::operator>(ProcessHandle const &o) const
{
	return compare(o) > 0;
}

bool ProcessHandle::operator>=(ProcessHandle const &o) const
{
	return compare(o) >= 0;
}
}
}
}

namespace
{
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

#ifdef _WIN32
bdrck::process::detail::ProcessHandle
launchProcess(bdrck::process::StandardStreamPipes &,
              bdrck::process::ProcessArguments const &)
{
	return INVALID_PROCESS_HANDLE_VALUE;
}
#else
[[noreturn]] void throwChildSignalError(int sig)
{
	char *message = ::strsignal(sig);
	if(message != nullptr)
		throw std::runtime_error(message);
	else
		throw std::runtime_error("Unrecognized signal.");
}

void replaceStdStream(bdrck::process::StandardStreamPipes const &pipes,
                      bdrck::process::StdStream stream)
{
	bdrck::process::PipeSide side = bdrck::process::PipeSide::WRITE;
	if(stream == bdrck::process::StdStream::STDIN)
		side = bdrck::process::PipeSide::READ;

	auto src = bdrck::process::pipe::pipeCastToNative(
	        pipes.at(stream).get(side));
	auto dst = bdrck::process::pipe::pipeCastToNative(
	        bdrck::process::pipe::getStreamPipe(stream));

	int ret = dup2(src, dst);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
}

bdrck::process::detail::ProcessHandle
launchProcess(bdrck::process::StandardStreamPipes &pipes,
              bdrck::process::ProcessArguments const &args)
{
	// Open a pipe, so we can get error messages from our child.
	bdrck::process::Pipe errorPipe(O_CLOEXEC);

	// Open pipes for the child's standard streams.
	bdrck::process::pipe::openPipes(pipes);

	// Fork a new process.

	pid_t pid = fork();
	if(pid == -1)
		bdrck::util::error::throwErrnoError();

	if(pid == 0)
	{
		// In the child process. Try to exec the binary.

		try
		{
			bdrck::process::pipe::close(
			        errorPipe, bdrck::process::PipeSide::READ);

			bdrck::process::pipe::closeParentSide(pipes);

			replaceStdStream(pipes,
			                 bdrck::process::StdStream::STDIN);
			replaceStdStream(pipes,
			                 bdrck::process::StdStream::STDOUT);
			replaceStdStream(pipes,
			                 bdrck::process::StdStream::STDERR);

			if(execvp(args.file, args.argv) == -1)
				bdrck::util::error::throwErrnoError();
		}
		catch(std::runtime_error const &e)
		{
			std::string message = e.what();
			std::size_t written = bdrck::process::pipe::write(
			        errorPipe.get(bdrck::process::PipeSide::WRITE),
			        message.c_str(), message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		catch(...)
		{
			std::string message = "Unknown error.";
			std::size_t written = bdrck::process::pipe::write(
			        errorPipe.get(bdrck::process::PipeSide::WRITE),
			        message.c_str(), message.length());
			assert(written ==
			       static_cast<ssize_t>(message.length()));
		}
		_exit(EXIT_FAILURE);
	}
	else
	{
		// Still in the parent process. Check for errors.

		bdrck::process::pipe::close(errorPipe,
		                            bdrck::process::PipeSide::WRITE);

		bdrck::process::pipe::closeChildSide(pipes);

		std::string error = bdrck::process::pipe::readAll(
		        errorPipe, bdrck::process::PipeSide::READ);
		bdrck::process::pipe::close(errorPipe,
		                            bdrck::process::PipeSide::READ);
		if(!error.empty())
			throw std::runtime_error(error);

		return pid;
	}
}
#endif

int waitOnProcessHandle(bdrck::process::detail::ProcessHandle &handle)
{
#ifdef _WIN32
	if(handle.handle == INVALID_HANDLE_VALUE)
		return EXIT_SUCCESS;

	DWORD waitResult = WaitForSingleObject(handle.handle, INFINITE);
	if(waitResult == WAIT_FAILED)
		throw std::runtime_error("Waiting for child process failed.");

	DWORD exitCode = 0;
	BOOL ret = GetExitCodeProcess(handle.handle, &exitCode);
	if(!ret)
	{
		throw std::runtime_error(
		        "Getting child process exit code failed.");
	}

	CloseHandle(handle.handle);
	handle.handle = INVALID_PROCESS_HANDLE_VALUE;

	return static_cast<int>(exitCode);
#else
	if(handle.handle == -1)
		return EXIT_SUCCESS;

	int status;
	while(waitpid(handle.handle, &status, 0) == -1)
	{
		if(errno != EINTR)
			bdrck::util::error::throwErrnoError();
	}

	handle.handle = INVALID_PROCESS_HANDLE_VALUE;

	if(WIFEXITED(status))
		return WEXITSTATUS(status);
	else if(WIFSIGNALED(status))
		throwChildSignalError(WTERMSIG(status));

	return EXIT_FAILURE;
#endif
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
          parent(std::make_unique<detail::ProcessHandle>(
                  getCurrentProcessHandle())),
          child(std::make_unique<detail::ProcessHandle>(
                  INVALID_PROCESS_HANDLE_VALUE)),
          pipes()
{
	*child = launchProcess(pipes, args);
}

Process::~Process()
{
	try
	{
		pipe::closeParentSide(pipes);
		wait();
	}
	catch(...)
	{
	}
}

PipeDescriptor Process::getPipe(StdStream stream) const
{
	switch(stream)
	{
	case StdStream::STDIN:
		return pipes.at(stream).get(PipeSide::WRITE);

	case StdStream::STDOUT:
	case StdStream::STDERR:
		return pipes.at(stream).get(PipeSide::READ);
	}
	return pipe::pipeCastFromNative(INVALID_PIPE_VALUE);
}

void Process::closePipe(StdStream stream)
{
	auto &pipe = pipes.at(stream);

	PipeSide side = PipeSide::READ;
	if(stream == StdStream::STDIN)
		side = PipeSide::WRITE;

	pipe::close(pipe, side);
	pipe.set(side, pipe::pipeCastFromNative(INVALID_PIPE_VALUE));
}

int Process::wait()
{
	return waitOnProcessHandle(*child);
}
}
}
