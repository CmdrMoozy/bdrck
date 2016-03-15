#include "deserialize.hpp"

#include <stdexcept>

namespace bdrck
{
namespace config
{
template <> bool deserialize(std::string const &serialized)
{
	if(serialized == "true")
		return true;
	else if(serialized == "false")
		return false;
	else
		throw std::runtime_error("Deserializing boolean failed.");
}
}
}
