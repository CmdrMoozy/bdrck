#ifndef bdrck_params_detail_parseCommand_HPP
#define bdrck_params_detail_parseCommand_HPP

#include <set>

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
struct ProgramParameters;

namespace detail
{
std::set<Command>::const_iterator
parseCommand(ProgramParameters &parameters,
             std::set<Command> const &commands) noexcept;
}
}
}

#endif
