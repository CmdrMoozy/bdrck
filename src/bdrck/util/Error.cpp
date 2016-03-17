#include "Error.hpp"

#include <cerrno>
#include <stdexcept>
#include <system_error>

#ifdef _WIN32
#include <Windows.h>
#endif

namespace bdrck
{
namespace util
{
namespace error
{
std::string getErrnoError(boost::optional<int> error,
                          std::string const &defaultMessage) noexcept
{
	if(!error)
		error = errno;

	std::string message = std::system_category().message(*error);
	if(message.find("Unknown error") == 0)
		return defaultMessage;
	return message;
}

[[noreturn]] void throwErrnoError(boost::optional<int> error,
                                  std::string const &defaultMessage)
{
	throw std::runtime_error(getErrnoError(error, defaultMessage));
}

#ifdef _WIN32
std::string getLastWindowsError()
{
	DWORD error = GetLastError();

	LPTSTR buffer;
	DWORD ret =
	        FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER, nullptr, error, 0,
	                      static_cast<LPTSTR>(&buffer), 0, nullptr);

	if(ret == 0)
		return std::string("Unknown error.");

	std::string errorMessage = lptstrToString(buffer);
	LocalFree(buffer);
	return errorMessage);
}

[[noreturn]] void throwLastWindowsError()
{
	throw std::runtime_error(getLastWindowsError());
}
#endif
}
}
}
