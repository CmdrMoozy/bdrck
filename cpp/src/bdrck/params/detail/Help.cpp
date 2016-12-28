#include "Help.hpp"

#include <iostream>

namespace bdrck
{
namespace params
{
namespace detail
{
void printProgramHelp(std::string const &program,
                      std::set<bdrck::params::Command> const &commands)
{
	std::cout << "Usage: " << program
	          << " command [options ...] [arguments ...]\n";
	std::cout << "Available commands:\n";
	for(auto const &command : commands)
	{
		std::cout << "\t" << command.name << " - " << command.help
		          << "\n";
	}
}

void printCommandHelp(std::string const &program,
                      bdrck::params::Command const &command,
                      bool printCommandName)
{
	std::cout << "Usage: " << program << " ";
	if(printCommandName)
		std::cout << command.name << " ";
	std::cout << "[options ...] [arguments ...]\n";

	if(command.options.size() > 0)
	{
		std::cout << "\nOptions:\n";
		for(auto const &option : command.options)
		{
			std::cout << "\t--" << option.name;
			if(!!option.shortName)
				std::cout << ", -" << *option.shortName;
			std::cout << " - " << option.help;

			if(option.isFlag)
			{
				std::cout << " [Flag, default: off]";
			}
			else if(!!option.defaultValue)
			{
				std::cout
				        << " [Default: " << *option.defaultValue
				        << "]";
			}
			std::cout << "\n";
		}
	}

	if(command.arguments.size() > 0)
	{
		std::cout << "\nPositional arguments:";
		for(auto const &argument : command.arguments)
		{
			std::cout << "\n\t" << argument.name << " - "
			          << argument.help;
			if(!!argument.defaultValue)
			{
				std::cout << " [Default: "
				          << *argument.defaultValue << "]";
			}
		}
		if(command.lastArgumentIsVariadic)
			std::cout << " [One or more]";
		std::cout << "\n";
	}
}
}
}
}
