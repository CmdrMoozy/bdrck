#include "parseCommand.hpp"

#include "bdrck/params/ProgramParameters.hpp"

namespace bdrck
{
namespace params
{
namespace detail
{
std::set<Command>::const_iterator
parseCommand(ProgramParameters &parameters,
             std::set<Command> const &commands) noexcept
{
	if(parameters.parameters.empty())
		return commands.cend();
	Command search(parameters.parameters.front(), "", CommandFunction());
	auto ret = commands.find(search);
	if(ret != commands.cend())
		parameters.parameters.pop_front();
	return ret;
}
}
}
}
