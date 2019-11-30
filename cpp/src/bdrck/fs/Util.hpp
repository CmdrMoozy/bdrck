#ifndef bdrck_fs_Util_HPP
#define bdrck_fs_Util_HPP

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

#include <boost/optional/optional.hpp>

namespace bdrck
{
namespace fs
{
typedef std::chrono::time_point<std::chrono::high_resolution_clock>
        FilesystemTime;

/*!
 * Normalize a path by converting to POSIX separators ('/') and removing any
 * trailing separators.
 *
 * \param p The original path.
 * \return The normalized path.
 */
std::string normalizePath(const std::string &p);

FilesystemTime lastWriteTime(std::string const &p);
void lastWriteTime(std::string const &p, FilesystemTime const &t);

/*!
 * Returns the system's default configuration path (optionally an
 * application-specific one).
 *
 * \param application The application, for an application-specific path.
 * \return The system's configuration path.
 */
std::string getConfigurationDirectoryPath(
        boost::optional<std::string> const &application = boost::none);
}
}

#endif
