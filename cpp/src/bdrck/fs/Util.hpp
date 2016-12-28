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
 * \param p A relative path to resolve.
 * \return A resolved absolute path that has been normalized.
 */
std::string resolvePath(std::string const &p);

std::string combinePaths(std::string const &a, std::string const &b);
std::string combinePaths(std::vector<std::string> const &c);
std::string combinePaths(std::string const &a,
                         std::vector<std::string> const &c);

/*!
 * \param p The input path.
 * \return The entire path, except the last component.
 */
std::string dirname(std::string const &p);

/*!
 * \param p The input path.
 * \return The last component of the path, with no separators.
 */
std::string basename(std::string const &p);

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

std::vector<std::string> glob(std::string const &pattern);

bool exists(const std::string &p);

bool isFile(std::string const &p);
bool isDirectory(std::string const &p);
bool isExecutable(std::string const &p);

void createFile(std::string const &p);

std::uintmax_t fileSize(std::string const &p);

FilesystemTime lastWriteTime(std::string const &p);
void lastWriteTime(std::string const &p, FilesystemTime const &t);

void copyFile(std::string const &src, std::string const &dst);
std::string readEntireFile(std::string const &p);

void removeFile(std::string const &p);

void createDirectory(std::string const &p);

/*!
 * \param p The path to the directory to remove.
 * \param recursive Whether or not to recursively remove contents.
 */
void removeDirectory(std::string const &p, bool recursive);

/*!
 * Create a directory path, including all necessary parent directories.
 *
 * \param p The path to create.
 */
void createPath(const std::string &p);

void createSymlink(std::string const &target, std::string const &link);

std::string getCurrentExecutable();
std::string getCurrentDirectory();

std::string getTemporaryDirectoryPath();

/*!
 * Returns the system's default configuration path (optionally an
 * application-specific one).
 *
 * \param application The application, for an application-specific path.
 * \return The system's configuration path.
 */
std::string getConfigurationDirectoryPath(
        boost::optional<std::string> const &application = boost::none);

boost::optional<std::string>
which(std::string const &command,
      boost::optional<std::string> const &hint = boost::none);
}
}

#endif
