#include "Terminal.hpp"

#include <cerrno>

#include <unistd.h>

#include "bdrck/util/Error.hpp"

namespace bdrck
{
namespace process
{
namespace terminal
{
int streamFD(StdStream stream)
{
	switch(stream)
	{
	case StdStream::In:
		return STDIN_FILENO;
	case StdStream::Out:
		return STDOUT_FILENO;
	case StdStream::Err:
		return STDERR_FILENO;
	}
}

bool isInteractiveTerminal(StdStream stream)
{
	int r = ::isatty(streamFD(stream));
	if(r == 0 && errno == EBADF)
		util::error::throwErrnoError();
	return r == 1;
}
}
}
}
