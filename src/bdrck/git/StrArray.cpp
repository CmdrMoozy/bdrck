#include "StrArray.hpp"

namespace bdrck
{
namespace git
{
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
