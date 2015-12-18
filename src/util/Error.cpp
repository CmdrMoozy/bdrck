#include "Error.hpp"

#include <cerrno>
#include <cstring>
#include <stdexcept>

namespace bdrck
{
namespace util
{
namespace error
{
[[noreturn]] void throwErrnoError(std::experimental::optional<int> error,
                                  std::string const &defaultMessage)
{
	if(!error)
		error = errno;
	char *message = std::strerror(*error);
	throw std::runtime_error(message != nullptr ? message
	                                            : defaultMessage.c_str());
}
}
}
}
