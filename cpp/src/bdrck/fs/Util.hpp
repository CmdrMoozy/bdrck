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

/*!
 * This function returns the path to the deepest directory which contains all
 * of the files or directories in the given list of paths. If no such path
 * exists (either because the list is empty, or because their paths have
 * nothing in common), then an empty string is returned instead.
 *
 * \param paths The list of paths to examine.
 * \return The path containing all the given paths.
 */
std::string commonParentPath(std::vector<std::string> const &paths);

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
