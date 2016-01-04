#include "Util.hpp"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <sstream>
#include <stdexcept>

#include <ftw.h>
#include <glob.h>
#include <unistd.h>
#include <linux/limits.h>
#include <sys/stat.h>
#include <sys/types.h>

#include "bdrck/algorithm/String.hpp"
#include "bdrck/cwrap/Unistd.hpp"
#include "bdrck/util/Error.hpp"

namespace
{
constexpr int FILE_TREE_WALK_OPEN_FDS = 128;

int removeDirectoryCallback(char const *p, struct stat const *, int t,
                            struct FTW *)
{
	switch(t)
	{
	case FTW_F:
	case FTW_SL:
	case FTW_SLN:
	{
		int ret = unlink(p);
		if(ret != 0)
			throw std::runtime_error(
			        "Removing directory contents failed.");
	}
	break;

	case FTW_D:
	case FTW_DP:
	{
		int ret = rmdir(p);
		if(ret != 0)
			throw std::runtime_error(
			        "Removing directory contents failed.");
	}
	break;

	case FTW_DNR:
	case FTW_NS:
	default:
		throw std::runtime_error("Removing directory contents failed.");
	}

	return FTW_CONTINUE;
}

struct GlobBuffer
{
	glob_t buffer;

	GlobBuffer(char const *pattern, int flags,
	           int (*errfunc)(char const *, int))
	        : buffer()
	{
		switch(glob(pattern, flags, errfunc, &buffer))
		{
		case 0:
		case GLOB_NOMATCH: // No matches isn't really an error.
			break;

		case GLOB_NOSPACE:
			bdrck::util::error::throwErrnoError(ENOMEM);

		case GLOB_ABORTED:
			bdrck::util::error::throwErrnoError(EIO);

		default:
			throw std::runtime_error("Unknown error.");
		}
	}

	~GlobBuffer()
	{
		globfree(&buffer);
	}
};
}

namespace bdrck
{
namespace fs
{
std::string normalizePath(const std::string &p)
{
	std::string ret = p;

	// Convert Windows-style separators to POSIX separators.
	std::transform(ret.begin(), ret.end(), ret.begin(),
	               [](const char &c) -> char
	               {
		               if(c == '\\')
			               return '/';
		               return c;
		       });

	// Remove any trailing separators.
	while(!ret.empty() && (*ret.rbegin() == '/'))
		ret.erase(ret.length() - 1);

	return ret;
}

std::string combinePaths(std::string const &a, std::string const &b)
{
	auto aEnd = a.find_last_not_of("\\/");
	auto bStart = b.find_first_not_of("\\/");

	std::ostringstream oss;
	if(aEnd != std::string::npos)
	{
		oss << a.substr(0, aEnd + 1);
	}
	else
	{
		// a must have been "/" (or an empty string). Prepend the root
		// directory to b to make a valid final path.
		oss << "/";
	}
	if((aEnd != std::string::npos) && (bStart != std::string::npos))
		oss << "/";
	if(bStart != std::string::npos)
		oss << b.substr(bStart);

	return oss.str();
}

std::string combinePaths(std::vector<std::string> const &c)
{
	if(c.empty())
		return "";
	if(c.size() == 1)
		return *c.begin();
	std::string ret = combinePaths(c[0], c[1]);
	for(std::size_t i = 2; i < c.size(); ++i)
		ret = combinePaths(ret, c[i]);
	return ret;
}

std::string combinePaths(std::string const &a,
                         std::vector<std::string> const &c)
{
	std::vector<std::string> components;
	components.reserve(c.size() + 1);
	components.emplace_back(a);
	for(auto const &component : c)
		components.emplace_back(component);
	return combinePaths(components);
}

std::string dirname(std::string const &p)
{
	std::string path = normalizePath(p);
	std::string::size_type lastSeparator = path.find_last_of('/');
	if(lastSeparator == std::string::npos)
		return path;
	return path.substr(0, lastSeparator);
}

std::string basename(std::string const &p)
{
	std::string path = normalizePath(p);
	std::string::size_type lastSeparator = path.find_last_of('/');
	if(lastSeparator == std::string::npos)
		return path;
	return path.substr(lastSeparator + 1);
}

std::vector<std::string> glob(std::string const &pattern)
{
	GlobBuffer buffer(pattern.c_str(), GLOB_ERR | GLOB_NOSORT, nullptr);
	std::vector<std::string> paths;
	for(std::size_t i = 0; i < buffer.buffer.gl_pathc; ++i)
		paths.emplace_back(buffer.buffer.gl_pathv[i]);
	return paths;
}

bool exists(const std::string &p)
{
	struct stat stats;
	int ret = stat(p.c_str(), &stats);
	return ret == 0;
}

bool isFile(std::string const &p)
{
	struct stat stats;
	int ret = stat(p.c_str(), &stats);
	if(ret != 0)
		return false;
	return S_ISREG(stats.st_mode);
}

bool isDirectory(std::string const &p)
{
	struct stat stats;
	int ret = stat(p.c_str(), &stats);
	if(ret != 0)
		return false;
	return S_ISDIR(stats.st_mode);
}

bool isExecutable(std::string const &p)
{
	int ret = access(p.c_str(), X_OK);
	return ret == 0;
}

void createFile(std::string const &p)
{
	FILE *f = fopen(p.c_str(), "a");
	if(f == nullptr)
		throw std::runtime_error("Creating file failed.");
	fclose(f);
}

void removeFile(std::string const &p)
{
	if(!exists(p))
		return;
	if(!isFile(p))
	{
		throw std::runtime_error(
		        "Cannot remove non-file paths with this function.");
	}
	int ret = std::remove(p.c_str());
	if(ret != 0)
		throw std::runtime_error("Removing file failed.");
}

void createDirectory(std::string const &p)
{
	if(isDirectory(p))
		return;
	int ret = mkdir(p.c_str(), 0777);
	if(ret != 0)
		throw std::runtime_error("Creating directory failed.");
}

void removeDirectory(std::string const &p)
{
	if(!exists(p))
		return;
	if(!isDirectory(p))
	{
		throw std::runtime_error("Cannot remove non-directory paths "
		                         "with this function.");
	}

	int ret = nftw(p.c_str(), removeDirectoryCallback,
	               FILE_TREE_WALK_OPEN_FDS,
	               FTW_ACTIONRETVAL | FTW_DEPTH | FTW_PHYS);
	if(ret != 0)
	{
		throw std::runtime_error("Removing directory contents failed.");
	}
}

void createPath(const std::string &p)
{
	std::vector<std::string> components =
	        bdrck::algorithm::string::split(normalizePath(p), '/');
	std::string currentPath = "";

	for(const auto &component : components)
	{
		currentPath = combinePaths(currentPath, component);
		if(isDirectory(currentPath))
			continue;
		if(!exists(currentPath))
			createDirectory(currentPath);
	}
}

std::string getCurrentExecutable()
{
	return bdrck::cwrap::unistd::readlink("/proc/self/exe");
}

std::string getCurrentDirectory()
{
	return dirname(getCurrentExecutable());
}

std::string getTemporaryDirectoryPath()
{
	std::string path("/tmp");

	char const *tmpdir = std::getenv("TMPDIR");
	if(tmpdir != nullptr)
	{
		std::string tmpdirStr(tmpdir);
		if(isDirectory(tmpdirStr))
			path = tmpdirStr;
	}

	return path;
}

std::string getConfigurationDirectoryPath()
{
	std::string path;
	std::string suffix;

	char *home = std::getenv("XDG_CONFIG_HOME");
	if(home == nullptr)
	{
		home = std::getenv("HOME");
		if(home == nullptr)
		{
			throw std::runtime_error(
			        "Couldn't find home directory.");
		}
		suffix.assign(".config");
	}
	path.assign(home);
	path = combinePaths(path, suffix);

	if(!exists(path))
		createDirectory(path);

	if(!isDirectory(path))
	{
		throw std::runtime_error(
		        "Configuration directory is not a directory.");
	}

	return path;
}

std::experimental::optional<std::string> which(std::string const &command)
{
	char const *p = std::getenv("PATH");
	std::string path;
	if(p != nullptr)
		path.assign(p);

	std::vector<std::string> pathComponents =
	        bdrck::algorithm::string::split(path, ':');
	for(auto const &directory : pathComponents)
	{
		std::string commandPath = combinePaths(directory, command);
		if(isExecutable(commandPath))
			return commandPath;
	}

	return std::experimental::nullopt;
}
}
}
