#include "Error.hpp"

#include <cerrno>
#include <stdexcept>
#include <system_error>

namespace bdrck
{
namespace util
{
namespace error
{
std::string getErrnoError(std::experimental::optional<int> error,
                          std::string const &defaultMessage) noexcept
{
	if(!error)
		error = errno;

	std::string message = std::system_category().message(*error);
	if(message.find("Unknown error") == 0)
		return defaultMessage;
	return message;
}

[[noreturn]] void throwErrnoError(std::experimental::optional<int> error,
                                  std::string const &defaultMessage)
{
	throw std::runtime_error(getErrnoError(error, defaultMessage));
}
}
}
}
