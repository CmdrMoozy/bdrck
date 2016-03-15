#include "serialize.hpp"

namespace bdrck
{
namespace config
{
std::string serialize(bool const &value)
{
	return value ? "true" : "false";
}
}
}
