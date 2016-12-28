#ifndef bdrck_params_detail_parseOptions_HPP
#define bdrck_params_detail_parseOptions_HPP

#include <tuple>

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
struct ProgramParameters;

namespace detail
{
std::tuple<OptionsMap, FlagsMap> parseOptions(ProgramParameters &parameters,
                                              Command const &command);
}
}
}

#endif
