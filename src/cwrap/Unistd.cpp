#include "Unistd.hpp"

#include <unistd.h>
#include <linux/limits.h>
#include <sys/types.h>

#include "bdrck/util/Error.hpp"

namespace bdrck
{
namespace cwrap
{
namespace unistd
{
std::string readlink(char const *path)
{
	char buffer[PATH_MAX];
	ssize_t length = ::readlink(path, buffer, PATH_MAX);
	if(length == -1)
		bdrck::util::error::throwErrnoError();
	return std::string(&buffer[0], static_cast<std::size_t>(length));
}
}
}
}
