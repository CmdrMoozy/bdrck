#ifndef bdrck_fs_Util_HPP
#define bdrck_fs_Util_HPP

#include <string>
#include <vector>

namespace bdrck
{
namespace fs
{
std::string normalizePath(const std::string &p);

std::string combinePaths(std::string const &a, std::string const &b);
std::string combinePaths(std::vector<std::string> const &c);
std::string combinePaths(std::string const &a,
                         std::vector<std::string> const &c);

std::string dirname(std::string const &p);

bool exists(const std::string &p);

bool isFile(std::string const &p);
bool isDirectory(std::string const &p);

void createFile(std::string const &p);
void removeFile(std::string const &p);
void createDirectory(std::string const &p);
void removeDirectory(std::string const &p);

void createPath(const std::string &p);

std::string getCurrentExecutable();
std::string getCurrentDirectory();

std::string getTemporaryDirectoryPath();
std::string getConfigurationDirectoryPath();
}
}

#endif
