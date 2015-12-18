#include "String.hpp"

#include <cstring>

#include "bdrck/util/Error.hpp"

namespace bdrck
{
namespace cwrap
{
namespace string
{
char *strdup(char const *s)
{
	char *copy = ::strdup(s);
	if(copy == nullptr)
		::bdrck::util::error::throwErrnoError();
	std::string ret(copy);
	return copy;
}

std::string strsignal(int sig)
{
	char *str = ::strsignal(sig);
	if(str != nullptr)
		return str;
	else
		return "Unrecognized signal.";
}
}
}
}
