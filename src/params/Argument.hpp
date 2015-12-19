#ifndef bdrck_params_Argument_HPP
#define bdrck_params_Argument_HPP

#include <string>
#include <experimental/optional>

namespace bdrck
{
namespace params
{
struct Argument
{
	std::string name;
	std::string help;
	std::experimental::optional<std::string> defaultValue;

	Argument(std::string const &n, std::string const &h,
	         std::experimental::optional<std::string> const &dv =
	                 std::experimental::nullopt);

	Argument(std::string const &n, std::string const &h,
	         std::string const &dv);
};
}
}

#endif
