#ifndef bdrck_params_Argument_HPP
#define bdrck_params_Argument_HPP

#include <string>

#include <boost/optional/optional.hpp>

namespace bdrck
{
namespace params
{
/*!
 * An argument is a positional argument. It must come after any Options the
 * command supports, and can have a default value if it is not specified by
 * the user explicitly.
 *
 * The final argument to a Command can be variadic (that is, it can accept more
 * than one value), but whether or not this is the case is a property of the
 * command, not of the argument.
 */
struct Argument
{
	std::string name;
	std::string help;
	boost::optional<std::string> defaultValue;

	/*!
	 * \param n The name of the argument. Used to get value in the command.
	 * \param h The help message for this argument.
	 * \param dv The default value for this argument, if any.
	 */
	Argument(std::string const &n, std::string const &h,
	         boost::optional<std::string> const &dv = boost::none);

	/*!
	 * This is an extra Constructor overload, which allows for string
	 * literals as default values.
	 *
	 * \param n The name of the argument. Used to get value in the command.
	 * \param h The help message for this argument.
	 * \param dv The default value for this argument.
	 */
	Argument(std::string const &n, std::string const &h,
	         std::string const &dv);
};
}
}

#endif
