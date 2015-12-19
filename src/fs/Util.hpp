#ifndef bdrck_fs_Util_HPP
#define bdrck_fs_Util_HPP

#include <string>

namespace bdrck
{
namespace fs
{
std::string normalizePath(const std::string &p);

std::string combinePaths(const std::string &a, const std::string &b);

bool exists(const std::string &p);

bool isFile(std::string const &p);
bool isDirectory(std::string const &p);

void createFile(std::string const &p);
void removeFile(std::string const &p);
void createDirectory(std::string const &p);
void removeDirectory(std::string const &p);

void createPath(const std::string &p);

std::string getTemporaryDirectoryPath();
std::string getConfigurationDirectoryPath();
}
}

#endif
