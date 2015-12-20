#include "parseAndExecuteCommand.hpp"

#include "bdrck/params/detail/parseAndExecuteImpl.hpp"

namespace bdrck
{
namespace params
{
int parseAndExecuteCommand(int argc, char const *const *argv,
                           std::set<Command> const &commands)
{
	return detail::parseAndExecuteImpl(argc, argv, commands,
	                                   /*printProgramHelp=*/true,
	                                   /*printCommandName=*/true);
}
}
}
