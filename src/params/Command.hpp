#ifndef bdrck_params_Command_HPP
#define bdrck_params_Command_HPP

#include <initializer_list>
#include <functional>
#include <map>
#include <set>
#include <string>
#include <vector>

#include "bdrck/params/Argument.hpp"
#include "bdrck/params/Option.hpp"

namespace bdrck
{
namespace params
{
typedef std::map<std::string, std::string> OptionsMap;
typedef std::map<std::string, bool> FlagsMap;
typedef std::map<std::string, std::vector<std::string>> ArgumentsMap;

typedef std::function<void(OptionsMap const &, FlagsMap const &,
                           ArgumentsMap const &)> CommandFunction;

struct Command
{
	std::string name;
	std::string help;
	CommandFunction function;
	OptionSet options;
	std::vector<Argument> arguments;
	bool lastArgumentIsVariadic;

	Command(std::string const &n, std::string const &h,
	        CommandFunction const &fn,
	        std::initializer_list<Option> const &o = {},
	        std::vector<Argument> const &a = {}, bool laiv = false);
};

bool operator<(Command const &a, Command const &b);
}
}

#endif
