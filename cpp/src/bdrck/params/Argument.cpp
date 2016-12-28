#include "Argument.hpp"

namespace bdrck
{
namespace params
{
Argument::Argument(std::string const &n, std::string const &h,
                   boost::optional<std::string> const &dv)
        : name(n), help(h), defaultValue(dv)
{
}

Argument::Argument(std::string const &n, std::string const &h,
                   std::string const &dv)
        : name(n), help(h), defaultValue(dv)
{
}
}
}
