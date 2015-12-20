#include "parseAndExecuteImpl.hpp"

#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <string>
#include <tuple>

#include "bdrck/params/ProgramParameters.hpp"
#include "bdrck/params/detail/Help.hpp"
#include "bdrck/params/detail/parseArguments.hpp"
#include "bdrck/params/detail/parseCommand.hpp"
#include "bdrck/params/detail/parseOptions.hpp"

namespace bdrck
{
namespace params
{
namespace detail
{
int parseAndExecuteImpl(int argc, char const *const *argv,
                        std::set<Command> const &commands,
                        bool printProgramHelp, bool printCommandName)
{
	ProgramParameters parameters(argc, argv);

	// First, figure out which command we'll be parsing parameters for.
	auto commandIt = detail::parseCommand(parameters, commands);
	if(commandIt == commands.cend())
	{
		if(printProgramHelp)
			detail::printProgramHelp(argv[0], commands);
		return EXIT_FAILURE;
	}

	// Parse this command's options and arguments.
	std::tuple<OptionsMap, FlagsMap> options;
	ArgumentsMap arguments;
	try
	{
		options = detail::parseOptions(parameters, *commandIt);
		arguments = detail::parseArguments(parameters, *commandIt);
	}
	catch(std::exception const &e)
	{
		std::cerr << "ERROR: " << e.what() << "\n";
		detail::printCommandHelp(argv[0], *commandIt, printCommandName);
		return EXIT_FAILURE;
	}
	catch(...)
	{
		std::cerr << "ERROR: Unknown exception\n";
		detail::printCommandHelp(argv[0], *commandIt, printCommandName);
		return EXIT_FAILURE;
	}

	// Execute the user-provided function.
	try
	{
		if(commandIt->function)
		{
			commandIt->function(std::get<0>(options),
			                    std::get<1>(options), arguments);
		}
		return EXIT_SUCCESS;
	}
	catch(std::exception const &e)
	{
		std::cerr << "ERROR: " << e.what() << "\n";
	}
	catch(...)
	{
		std::cerr << "ERROR: Unknown exception.\n";
	}

	return EXIT_FAILURE;
}
}
}
}
