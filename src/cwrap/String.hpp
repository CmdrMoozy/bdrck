#ifndef bdrck_cwrap_String_HPP
#define bdrck_cwrap_String_HPP

#include <string>

namespace bdrck
{
namespace cwrap
{
namespace string
{
/*!
 * A version of strdup() which is guaranteed to return a valid pointer. If the
 * copy could not be allocated, then an exception is thrown instead.
 *
 * \param s The string to duplicate.
 * \return The copy of the input string.
 */
char *strdup(char const *s);

/*!
 * \param sig The signal to describe.
 * \return The string describing the given signal.
 */
std::string strsignal(int sig);
}
}
}

#endif
