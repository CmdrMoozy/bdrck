#ifndef bdrck_params_detail_parseArguments_HPP
#define bdrck_params_detail_parseArguments_HPP

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
struct ProgramParameters;

namespace detail
{
ArgumentsMap parseArguments(ProgramParameters &parameters,
                            Command const &command);
}
}
}

#endif
