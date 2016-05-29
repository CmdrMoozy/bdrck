#include "Util.hpp"

#include <vector>

namespace bdrck
{
namespace git
{
std::string oidToString(git_oid const &oid)
{
	std::vector<char> buffer(40);
	git_oid_fmt(buffer.data(), &oid);
	return std::string(buffer.begin(), buffer.end());
}
}
}
