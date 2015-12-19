#ifndef bdrck_params_parseAndExecuteCommand_HPP
#define bdrck_params_parseAndExecuteCommand_HPP

#include <set>

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
int parseAndExecuteCommand(int argc, char const *const *argv,
                           std::set<Command> const &commands);
}
}

#endif
