#ifndef bdrck_fs_Util_HPP
#define bdrck_fs_Util_HPP

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>
#include <experimental/optional>

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
std::string getConfigurationDirectoryPath();

std::experimental::optional<std::string> which(std::string const &command);
}
}

#endif
