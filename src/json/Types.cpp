#include "Types.hpp"

namespace bdrck
{
namespace json
{
NullType::NullType()
{
}

bool NullType::operator==(NullType const &) const
{
	return true;
}

bool NullType::operator!=(NullType const &) const
{
	return false;
}

bool NullType::operator<(NullType const &) const
{
	return false;
}

bool NullType::operator<=(NullType const &) const
{
	return true;
}

bool NullType::operator>(NullType const &) const
{
	return false;
}

bool NullType::operator>=(NullType const &) const
{
	return true;
}

bdrck::string::StringRef toString(StringType const &s)
{
	return bdrck::string::StringRef(
	        reinterpret_cast<char const *>(s.data()), s.size());
}

StringType fromString(bdrck::string::StringRef const &s)
{
	return StringType(s.begin(), s.end());
}
}
}
