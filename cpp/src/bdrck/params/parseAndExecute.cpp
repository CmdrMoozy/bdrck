#include "parseAndExecute.hpp"

#include <vector>

#include "bdrck/params/detail/parseAndExecuteImpl.hpp"

namespace bdrck
{
namespace params
{
int parseAndExecute(int argc, char const *const *argv, Command const &command)
{
	std::vector<char const *> modifiedArgv;
	modifiedArgv.emplace_back(argv[0]);
	modifiedArgv.emplace_back(command.name.c_str());
	for(int i = 1; i < argc; ++i)
		modifiedArgv.emplace_back(argv[i]);

	int newArgc = argc + 1;
	char const *const *newArgv = modifiedArgv.data();

	return detail::parseAndExecuteImpl(newArgc, newArgv, {command},
	                                   /*printProgramHelp=*/false,
	                                   /*printCommandName=*/false);
}
}
}
