#include "ProgramParameters.hpp"

namespace bdrck
{
namespace params
{
ProgramParameters::ProgramParameters(std::list<std::string> const &p)
        : parameters(p)
{
}

ProgramParameters::ProgramParameters(
        std::initializer_list<std::string> const &p)
        : parameters(p)
{
}

ProgramParameters::ProgramParameters(int argc, char const *const *argv)
        : parameters()
{
	for(int i = 1; i < argc; ++i)
		parameters.emplace_back(argv[i]);
}
}
}
