#include "Command.hpp"

#include <algorithm>
#include <stdexcept>

namespace bdrck
{
namespace params
{
Command::Command(std::string const &n, std::string const &h,
                 CommandFunction const &fn,
                 std::initializer_list<Option> const &o,
                 std::vector<Argument> const &a, bool laiv)
        : name(n),
          help(h),
          function(fn),
          options(o),
          arguments(a),
          lastArgumentIsVariadic(laiv)
{
	// After the first argument with a default value, all other arguments
	// must also have default values (just like in C++).
	auto firstDefault = std::find_if(arguments.begin(), arguments.end(),
	                                 [](Argument const &argument) -> bool
	                                 {
		                                 return !!argument.defaultValue;
		                         });
	auto lastNonDefault =
	        std::find_if(arguments.rbegin(), arguments.rend(),
	                     [](Argument const &argument) -> bool
	                     {
		                     return !argument.defaultValue;
		             })
	                .base();
	if(lastNonDefault != arguments.end())
		--lastNonDefault;

	if(firstDefault < lastNonDefault)
	{
		throw std::runtime_error("Invalid command; after the first "
		                         "argument with a default value, all "
		                         "other arguments must also have "
		                         "default values.");
	}
}

bool operator<(Command const &a, Command const &b)
{
	return a.name < b.name;
}
}
}
