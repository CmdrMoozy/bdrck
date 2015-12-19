#include "parseArguments.hpp"

#include <sstream>
#include <stdexcept>

#include "bdrck/params/ProgramParameters.hpp"

namespace bdrck
{
namespace params
{
namespace detail
{
ArgumentsMap parseArguments(ProgramParameters &parameters,
                            Command const &command)
{
	ArgumentsMap retArguments;

	// Grab exactly one value for each argument, or until the parameters
	// list is empty.
	decltype(command.arguments)::const_iterator lastUnparsed;
	for(auto it = command.arguments.cbegin();
	    it != command.arguments.cend(); ++it)
	{
		lastUnparsed = it;
		if(parameters.parameters.empty())
			break;
		retArguments[it->name].push_back(parameters.parameters.front());
		parameters.parameters.pop_front();
		++lastUnparsed;
	}

	// If there were arguments we didn't get values for, insert their
	// default values.
	for(; lastUnparsed != command.arguments.cend(); ++lastUnparsed)
	{
		if(!!lastUnparsed->defaultValue)
		{
			retArguments[lastUnparsed->name].push_back(
			        *lastUnparsed->defaultValue);
		}
		else
		{
			std::ostringstream oss;
			oss << "No specified or default value for argument '"
			    << lastUnparsed->name << "'.";
			throw std::runtime_error(oss.str());
		}
	}

	// If the last argument is variadic, and there are any other parameters
	// left over, add them all to that last argument.
	if(command.lastArgumentIsVariadic)
	{
		std::string const &name = command.arguments.rbegin()->name;
		while(!parameters.parameters.empty())
		{
			retArguments[name].push_back(
			        parameters.parameters.front());
			parameters.parameters.pop_front();
		}
	}

	// If there are any parameters left over, we have a problem.
	if(!parameters.parameters.empty())
	{
		throw std::runtime_error("Found unused program parameters "
		                         "after parsing command parameters.");
	}

	return retArguments;
}
}
}
}
