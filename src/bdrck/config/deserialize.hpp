#ifndef bdrck_config_Deserializer_HPP
#define bdrck_config_Deserializer_HPP

#include <sstream>
#include <string>

namespace bdrck
{
namespace config
{
template <typename T> T deserialize(std::string const &serialized)
{
	T value;
	std::istringstream iss(serialized);
	iss >> value;
	return value;
}

template <> bool deserialize(std::string const &serialized);
}
}

#endif
