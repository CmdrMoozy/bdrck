#ifndef bdrck_cwrap_Unistd_HPP
#define bdrck_cwrap_Unistd_HPP

#include <string>

namespace bdrck
{
namespace cwrap
{
namespace unistd
{
/*!
 * Read the value of a symbolic link.
 *
 * \param path The path to the symbolic link.
 * \return The path the link points to.
 */
std::string readlink(char const *path);
}
}
}

#endif
