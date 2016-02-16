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
}
}
}

#endif
