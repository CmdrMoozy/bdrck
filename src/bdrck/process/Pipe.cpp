#include "Pipe.hpp"

#include <algorithm>
#include <cerrno>
#include <cstddef>
#include <sstream>
#include <stdexcept>
#include <vector>

#include "bdrck/process/PipeCast.hpp"
#include "bdrck/util/Error.hpp"

#ifdef _WIN32
#include <Windows.h>
#include <io.h>
#else
#include <fcntl.h>
#include <unistd.h>
#endif

namespace
{
constexpr std::size_t READ_BUFFER_SIZE = 256;
}

namespace bdrck
{
namespace process
{
namespace detail
{
struct PipeImpl
{
	NativePipe read;
	NativePipe write;

	PipeImpl(int flags = 0);

	PipeImpl(PipeImpl const &) = default;
	PipeImpl(PipeImpl &&) = default;
	PipeImpl &operator=(PipeImpl const &) = default;
	PipeImpl &operator=(PipeImpl &&) = default;

	~PipeImpl() = default;
};

PipeImpl::PipeImpl(int
#ifndef _WIN32
                           flags
#endif
                   )
{
#ifdef _WIN32
	SECURITY_ATTRIBUTES sattr;
	sattr.nLength = sizeof(SECURITY_ATTRIBUTES);
	sattr.bInheritHandle = TRUE;
	sattr.lpSecurityDescriptor = nullptr;

	BOOL ret = CreatePipe(&read, &write, &sattr, 0);
	if(!ret)
		throw std::runtime_error("Constructing pipe failed.");
#else
	int pipefd[2];
	int ret = pipe2(pipefd, flags);
	if(ret == -1)
		util::error::throwErrnoError();
	read = pipefd[0];
	write = pipefd[1];
#endif
}
}

Pipe::Pipe() : impl(std::make_unique<detail::PipeImpl>())
{
}

#ifndef _WIN32
Pipe::Pipe(int flags) : impl(std::make_unique<detail::PipeImpl>(flags))
{
}
#endif

Pipe::Pipe(Pipe const &o) : impl(std::make_unique<detail::PipeImpl>(*o.impl))
{
}

Pipe &Pipe::operator=(Pipe const &o)
{
	if(this == &o)
		return *this;
	impl = std::make_unique<detail::PipeImpl>(*o.impl);
	return *this;
}

Pipe::~Pipe()
{
}

PipeDescriptor Pipe::get(PipeSide side) const
{
	switch(side)
	{
	case PipeSide::READ:
		return pipe::pipeCastFromNative(impl->read);
	case PipeSide::WRITE:
		return pipe::pipeCastFromNative(impl->write);
	}
	return pipe::pipeCastFromNative(INVALID_PIPE_VALUE);
}

void Pipe::set(PipeSide side, PipeDescriptor descriptor)
{
	switch(side)
	{
	case PipeSide::READ:
		impl->read = pipe::pipeCastToNative(descriptor);
		break;

	case PipeSide::WRITE:
		impl->write = pipe::pipeCastToNative(descriptor);
		break;
	}
}

namespace pipe
{
PipeDescriptor getStreamPipe(StdStream stream)
{
	switch(stream)
	{
	case StdStream::STDIN:
		return 0;
	case StdStream::STDOUT:
		return 1;
	case StdStream::STDERR:
		return 2;
	}
	return pipeCastFromNative(INVALID_PIPE_VALUE);
}

bool isInteractiveTerminal(PipeDescriptor pipe)
{
	// _isatty accepts an integer file descriptor on Windows, not a
	// HANDLE like most other pipe-related functions.
	int fd = static_cast<int>(pipe);

#ifdef _WIN32
	int r = _isatty(fd);
#else
	int r = ::isatty(fd);
#endif
	if(r == 0 && errno == EBADF)
		util::error::throwErrnoError();
	return r == 1;
}

void openPipes(StandardStreamPipes &pipes)
{
	pipes.emplace(
	        std::make_pair<StdStream, Pipe>(StdStream::STDIN, Pipe()));
	pipes.emplace(
	        std::make_pair<StdStream, Pipe>(StdStream::STDOUT, Pipe()));
	pipes.emplace(
	        std::make_pair<StdStream, Pipe>(StdStream::STDERR, Pipe()));
}

std::string read(PipeDescriptor const &pipe, std::size_t count)
{
	std::vector<char> buffer(READ_BUFFER_SIZE);
	std::ostringstream oss;

	while(count > 0)
	{
#ifdef _WIN32

		DWORD bytesRead = 0;
		BOOL ret = ReadFile(pipeCastToNative(pipe), buffer.data(),
		                    static_cast<DWORD>(buffer.size()),
		                    &bytesRead, nullptr);
		if(!ret)
			throw std::runtime_error("Reading from pipe failed.");
		if(bytesRead == 0)
			break;

		oss << std::string(buffer.data(),
		                   static_cast<std::size_t>(bytesRead));
		count -= static_cast<std::size_t>(bytesRead);
#else
		ssize_t readCount =
		        ::read(pipeCastToNative(pipe), buffer.data(),
		               std::min(buffer.size(), count));
		if(readCount == -1)
			bdrck::util::error::throwErrnoError();
		if(readCount == 0)
			break;

		oss << std::string(buffer.data(),
		                   static_cast<std::size_t>(readCount));
		count -= static_cast<std::size_t>(readCount);
#endif
	}

	return oss.str();
}

std::string read(Pipe const &pipe, PipeSide side, std::size_t count)
{
	return read(pipe.get(side), count);
}

std::string readAll(PipeDescriptor const &pipe)
{
#ifdef _WIN32
	std::vector<char> buffer(READ_BUFFER_SIZE);
	std::ostringstream oss;
	DWORD bytesRead = 0;
	BOOL ret = FALSE;
	while(true)
	{
		ret = ReadFile(pipeCastToNative(pipe), buffer.data(),
		               static_cast<DWORD>(buffer.size()), &bytesRead,
		               nullptr);
		if(ret && bytesRead == 0)
			break;
		if(!ret)
			throw std::runtime_error("Reading from pipe failed.");
		oss << std::string(buffer.data(),
		                   static_cast<std::size_t>(bytesRead));
	}
	return oss.str();
#else
	std::vector<char> buffer(READ_BUFFER_SIZE);
	std::ostringstream oss;
	ssize_t count;
	while((count = ::read(pipeCastToNative(pipe), buffer.data(),
	                      buffer.size())) != 0)
	{
		if(count == -1)
			bdrck::util::error::throwErrnoError();
		oss << std::string(buffer.data(),
		                   static_cast<std::size_t>(count));
	}
	return oss.str();
#endif
}

std::string readAll(Pipe const &pipe, PipeSide side)
{
	return readAll(pipe.get(side));
}

std::size_t write(PipeDescriptor const &pipe, void const *buffer,
                  std::size_t size)
{
#ifdef _WIN32
	DWORD bytesWritten = 0;
	BOOL ret = WriteFile(pipeCastToNative(pipe), buffer,
	                     static_cast<DWORD>(size), &bytesWritten, nullptr);
	if(!ret)
		throw std::runtime_error("Writing to pipe failed.");
	return static_cast<std::size_t>(bytesWritten);
#else
	ssize_t written = ::write(pipeCastToNative(pipe), buffer, size);
	if(written == -1)
		bdrck::util::error::throwErrnoError();
	return static_cast<std::size_t>(written);
#endif
}

std::size_t write(Pipe const &pipe, PipeSide side, void const *buffer,
                  std::size_t size)
{
	return write(pipe.get(side), buffer, size);
}

void close(PipeDescriptor const &pipe)
{
	auto pipeDescriptor = pipeCastToNative(pipe);
	if(pipeDescriptor == INVALID_PIPE_VALUE)
		return;

#ifdef _WIN32
	BOOL ret = CloseHandle(pipeDescriptor);
	if(!ret)
		throw std::runtime_error("Closing pipe failed.");
#else
	int ret = ::close(pipeDescriptor);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
#endif
}

void close(Pipe const &pipe, PipeSide side)
{
	close(pipe.get(side));
}

void closeParentSide(StandardStreamPipes const &pipes)
{
	close(pipes.at(bdrck::process::StdStream::STDIN), PipeSide::WRITE);
	close(pipes.at(bdrck::process::StdStream::STDOUT), PipeSide::READ);
	close(pipes.at(bdrck::process::StdStream::STDERR), PipeSide::READ);
}

void closeChildSide(StandardStreamPipes const &pipes)
{
	close(pipes.at(bdrck::process::StdStream::STDIN), PipeSide::READ);
	close(pipes.at(bdrck::process::StdStream::STDOUT), PipeSide::WRITE);
	close(pipes.at(bdrck::process::StdStream::STDERR), PipeSide::WRITE);
}
}
}
}
