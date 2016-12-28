#include "StrArray.hpp"

namespace bdrck
{
namespace git
{
StrArray::StrArray(std::initializer_list<std::string> const &s)
        : StrArray(s.begin(), s.end())
{
}

git_strarray &StrArray::get()
{
	return strarray;
}

git_strarray const &StrArray::get() const
{
	return strarray;
}
}
}
