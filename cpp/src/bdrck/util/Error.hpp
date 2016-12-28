#ifndef bdrck_util_Error_HPP
#define bdrck_util_Error_HPP

#include <string>

#include <boost/optional/optional.hpp>

namespace bdrck
{
namespace util
{
namespace error
{
std::string
getErrnoError(boost::optional<int> error = boost::none,
              std::string const &defaultMessage = "Unknown error.") noexcept;

[[noreturn]] void
throwErrnoError(boost::optional<int> error = boost::none,
                std::string const &defaultMessage = "Unknown error.");

#ifdef _WIN32
/*!
 * This function is a wrapper for Windows' GetLastError() function, which
 * returns the last error message as a human-readable string.
 *
 * \return The last error as a human-readable string.
 */
std::string getLastWindowsError();

/*!
 * Throw an exception, with the last Windows error message as its "what"
 * parameter. See getLastWindowsError() for details.
 */
[[noreturn]] void throwLastWindowsError();
#endif
}
}
}

#endif
