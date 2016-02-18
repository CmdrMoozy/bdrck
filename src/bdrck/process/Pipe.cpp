#include "Pipe.hpp"

#include <cstddef>
#include <sstream>
#include <stdexcept>
#include <vector>

#include "bdrck/process/PipeCast.hpp"
#include "bdrck/util/Error.hpp"

#ifdef _WIN32
#include <Windows.h>
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

namespace pipe
{
void openPipes(StandardStreamPipes &pipes)
{
	pipes.emplace(std::make_pair<terminal::StdStream, Pipe>(
	        terminal::StdStream::In, Pipe()));
	pipes.emplace(std::make_pair<terminal::StdStream, Pipe>(
	        terminal::StdStream::Out, Pipe()));
	pipes.emplace(std::make_pair<terminal::StdStream, Pipe>(
	        terminal::StdStream::Err, Pipe()));
}

std::string readAll(Pipe const &pipe, PipeSide side)
{
#ifdef _WIN32
	std::vector<char> buffer(READ_BUFFER_SIZE);
	std::ostringstream oss;
	DWORD bytesRead = 0;
	BOOL ret = FALSE;
	while(true)
	{
		ret = ReadFile(pipeCastToNative(pipe.get(side)), buffer.data(),
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
	while((count = read(pipeCastToNative(pipe.get(side)), buffer.data(),
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

void closePipe(Pipe const &pipe, PipeSide side)
{
	auto pipeDescriptor = pipeCastToNative(pipe.get(side));
	if(pipeDescriptor == INVALID_PIPE_VALUE)
		return;

#ifdef _WIN32
	BOOL ret = CloseHandle(pipeDescriptor);
	if(!ret)
		throw std::runtime_error("Closing pipe failed.");
#else
	int ret = close(pipeDescriptor);
	if(ret == -1)
		bdrck::util::error::throwErrnoError();
#endif
}

void closeParentSide(StandardStreamPipes const &pipes)
{
	closePipe(pipes.at(bdrck::process::terminal::StdStream::In),
	          PipeSide::WRITE);
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Out),
	          PipeSide::READ);
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Err),
	          PipeSide::READ);
}

void closeChildSide(StandardStreamPipes const &pipes)
{
	closePipe(pipes.at(bdrck::process::terminal::StdStream::In),
	          PipeSide::READ);
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Out),
	          PipeSide::WRITE);
	closePipe(pipes.at(bdrck::process::terminal::StdStream::Err),
	          PipeSide::WRITE);
}
}
}
}
