#include "checkReturn.hpp"

#include <stdexcept>
#include <string>

#include <git2.h>

namespace bdrck
{
namespace git
{
void checkReturn(int r)
{
	if(r == 0)
		return;
	git_error const *err = giterr_last();
	if(err == nullptr)
		return;
	std::string errMsg(err->message);
	giterr_clear();
	throw std::runtime_error(errMsg);
}
}
}
