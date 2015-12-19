#ifndef bdrck_params_ProgramParameters_HPP
#define bdrck_params_ProgramParameters_HPP

#include <initializer_list>
#include <list>
#include <string>

namespace bdrck
{
namespace params
{
struct ProgramParameters
{
	std::list<std::string> parameters;

	explicit ProgramParameters(std::list<std::string> const &p);
	explicit ProgramParameters(std::initializer_list<std::string> const &p);
	ProgramParameters(int argc, char const *const *argv);
};
}
}

#endif
