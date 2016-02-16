#ifndef bdrck_params_detail_parseAndExecuteImpl_HPP
#define bdrck_params_detail_parseAndExecuteImpl_HPP

#include <set>

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
namespace detail
{
int parseAndExecuteImpl(int argc, char const *const *argv,
                        std::set<Command> const &commands,
                        bool printProgramHelp, bool printCommandName);
}
}
}

#endif
