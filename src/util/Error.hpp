#ifndef bdrck_util_Error_HPP
#define bdrck_util_Error_HPP

#include <string>
#include <experimental/optional>

namespace bdrck
{
namespace util
{
namespace error
{
[[noreturn]] void throwErrnoError(
        std::experimental::optional<int> error = std::experimental::nullopt,
        std::string const &defaultMessage = "Unknown error.");
}
}
}

#endif
