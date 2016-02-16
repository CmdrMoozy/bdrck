#include "Terminal.hpp"

#include <cerrno>

#include "bdrck/util/Error.hpp"

#ifdef _WIN32
#include <io.h>
#else
#include <unistd.h>
#endif

namespace bdrck
{
namespace process
{
namespace terminal
{
int streamFileno(StdStream stream)
{
	switch(stream)
	{
	case StdStream::In:
		return 0;
	case StdStream::Out:
		return 1;
	case StdStream::Err:
		return 2;
	}
	return -1;
}

bool isInteractiveTerminal(StdStream stream)
{
#ifdef _WIN32
	int r = _isatty(streamFileno(stream));
#else
	int r = ::isatty(streamFileno(stream));
#endif
	if(r == 0 && errno == EBADF)
		util::error::throwErrnoError();
	return r == 1;
}
}
}
}
