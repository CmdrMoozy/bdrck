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

/*!
 * A command is a "subcommand" for the overall executable. Examples of
 * applications which use "subcommands" include Git, Docker, and etc. If your
 * executable has only a single logical function, then a single command can
 * be constructed with an arbitrary name.
 */
struct Command
{
	std::string name;
	std::string help;
	CommandFunction function;
	OptionSet options;
	std::vector<Argument> arguments;
	bool lastArgumentIsVariadic;

	/*!
	 * \param n The name of the command, used to call it.
	 * \param h The help message for this command.
	 * \param fn The function to call when this command is executed.
	 * \param o The list of options this command accepts, if any.
	 * \param a The list of arguments this command accepts, if any.
	 * \param laiv Whether or not the last argument accepts >1 value.
	 */
	Command(std::string const &n, std::string const &h,
	        CommandFunction const &fn,
	        std::initializer_list<Option> const &o = {},
	        std::vector<Argument> const &a = {}, bool laiv = false);
};

bool operator<(Command const &a, Command const &b);
}
}

#endif
