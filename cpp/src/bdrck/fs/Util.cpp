#include "Util.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <functional>
#include <iterator>
#include <limits>
#include <locale>
#include <memory>
#include <numeric>
#include <sstream>
#include <stdexcept>

#include <sys/stat.h>
#include <sys/types.h>

#include <boost/filesystem.hpp>

#include "bdrck/algorithm/String.hpp"
#include "bdrck/util/Error.hpp"
#include "bdrck/util/ScopeExit.hpp"
#include "bdrck/util/Windows.hpp"

#ifdef _WIN32
#include <ShlObj.h>
#else
#include <fcntl.h>
#include <glob.h>
#include <linux/limits.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#endif

namespace
{
#ifdef _WIN32
// Windows epoch is 1601-01-01T00:00:00Z, or in seconds before UNIX epoch:
constexpr uint64_t WINDOWS_EPOCH_OFFSET_SEC = 11644473600ULL;

// Windows FILETIME timestamps are in units of 100 nanoseconds.
constexpr uint64_t WINDOWS_TICKS_PER_SECOND = 10000000ULL;

typedef std::chrono::duration<int64_t, std::ratio<1, WINDOWS_TICKS_PER_SECOND>>
        WindowsFiletimeDuration;
#endif

#ifdef _WIN32
class GlobState
{
public:
	GlobState(std::string const &p)
	        : pattern(p),
	          data(),
	          findHandle(INVALID_HANDLE_VALUE),
	          empty(false)
	{
		init();
	}

	~GlobState()
	{
		clear();
	}

	void forEachPath(std::function<void(std::string const &)> callback)
	{
		if(empty)
			return;

		BOOL foundNext = 0;
		do
		{
			callback(std::string(data.cFileName));
			foundNext = FindNextFile(findHandle, &data);
			if(!foundNext)
			{
				DWORD error = GetLastError();
				if(error == ERROR_NO_MORE_FILES)
				{
					init();
					return;
				}
				else
				{
					throw std::runtime_error(
					        "Iterating over globbed files "
					        "failed.");
				}
			}
		} while(foundNext);
	}

private:
	std::string pattern;
	WIN32_FIND_DATA data;
	HANDLE findHandle;
	bool empty;

	void clear()
	{
		if(findHandle != INVALID_HANDLE_VALUE)
			FindClose(findHandle);
	}

	void init()
	{
		if(empty)
			return;

		clear();
		findHandle = FindFirstFile(pattern.c_str(), &data);

		if(findHandle == INVALID_HANDLE_VALUE)
		{
			DWORD error = GetLastError();
			if(error == ERROR_FILE_NOT_FOUND)
			{
				empty = true;
			}
			else
			{
				throw std::runtime_error(
				        "Evaluating glob pattern failed.");
			}
		}
	}
};
#else
class GlobState
{
public:
	GlobState(std::string const &pattern) : buffer()
	{
		switch(glob(pattern.c_str(), GLOB_ERR | GLOB_NOSORT, nullptr,
		            &buffer))
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

	~GlobState()
	{
		globfree(&buffer);
	}

	void forEachPath(std::function<void(std::string const &)> callback)
	{
		for(std::size_t i = 0; i < buffer.gl_pathc; ++i)
			callback(buffer.gl_pathv[i]);
	}

private:
	glob_t buffer;
};
#endif
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
	               [](const char &c) -> char {
		               if(c == '\\')
			               return '/';
		               return c;
		       });

	// Remove any trailing separators.
	while(!ret.empty() && (*ret.rbegin() == '/'))
		ret.erase(ret.length() - 1);

	return ret;
}

std::string resolvePath(std::string const &p)
{
	boost::system::error_code ec;
	boost::filesystem::path resolved = boost::filesystem::canonical(p, ec);
	if(ec)
		throw std::runtime_error(ec.message());

	return normalizePath(resolved.string());
}

std::string combinePaths(std::string const &a, std::string const &b)
{
	if(a.length() == 0)
		return b;

	auto aEnd = a.find_last_not_of("\\/");
	auto bStart = b.find_first_not_of("\\/");

	std::ostringstream oss;
	if(aEnd != std::string::npos)
	{
		// If a is not an empty string or the root directory, add it
		// to the result (excluding the last /, if any).
		oss << a.substr(0, aEnd + 1);
	}

	if(a.length() > 0)
	{
		// If a was nonempty, add a slash to separate a from b.
		oss << "/";
	}

	// If b wasn't just a "/", then append it to the result.
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

std::string commonParentPath(std::vector<std::string> const &paths)
{
	if(paths.empty())
		return std::string();

	std::size_t minimumSize = std::numeric_limits<std::size_t>::max();
	std::accumulate(paths.begin(), paths.end(), minimumSize,
	                [](std::size_t minimum, std::string const &str) {
		                return std::min(minimum, str.length());
		        });

	char const *refStart = paths.back().c_str();
	char const *refEnd = refStart + minimumSize;

	while(refStart != refEnd)
	{
		bool same = true;
		for(auto it = paths.cbegin(); it != paths.cend() - 1; ++it)
		{
			if(!std::equal(refStart, refEnd, it->data()))
			{
				same = false;
				break;
			}
		}

		if(same)
			break;

		--refEnd;
	}

	if(refStart == refEnd)
		return std::string();
	return std::string(refStart, refEnd);
}

std::vector<std::string> glob(std::string const &pattern)
{
	GlobState state(pattern);
	std::vector<std::string> paths;
	state.forEachPath([&paths](std::string const &path) {
		paths.emplace_back(path);
	});
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
	boost::system::error_code ec;
	bool ret = boost::filesystem::is_regular_file(p, ec);
	if(ec)
	{
		if(ec.value() == boost::system::errc::no_such_file_or_directory)
		{
			return false;
		}

		throw std::runtime_error(ec.message());
	}
	return ret;
}

bool isDirectory(std::string const &p)
{
	boost::system::error_code ec;
	bool ret = boost::filesystem::is_directory(p, ec);
	if(ec)
	{
		if(ec.value() == boost::system::errc::no_such_file_or_directory)
		{
			return false;
		}

		throw std::runtime_error(ec.message());
	}
	return ret;
}

bool isExecutable(std::string const &p)
{
#ifdef _WIN32
	(void)p;
	return true;
#else
	int ret = access(p.c_str(), X_OK);
	return ret == 0;
#endif
}

void createFile(std::string const &p)
{
	std::ofstream out(p, std::ios_base::ate | std::ios_base::out);
}

std::uintmax_t fileSize(std::string const &p)
{
	struct stat stats;
	int ret = stat(p.c_str(), &stats);
	if(ret != 0)
		bdrck::util::error::throwErrnoError();
	return static_cast<std::uintmax_t>(stats.st_size);
}

FilesystemTime lastWriteTime(std::string const &p)
{
#ifdef _WIN32
	HANDLE file =
	        CreateFile(p.c_str(), GENERIC_READ, FILE_SHARE_WRITE, nullptr,
	                   OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
	if(file == INVALID_HANDLE_VALUE)
		throw std::runtime_error("Opening file handle failed.");
	bdrck::util::ScopeExit cleanup([&file]() { CloseHandle(file); });

	FILETIME writeTime;
	BOOL ret = GetFileTime(file, nullptr, nullptr, &writeTime);
	if(!ret)
		throw std::runtime_error("Getting file write time failed.");

	uint64_t timestamp =
	        (static_cast<uint64_t>(writeTime.dwHighDateTime) << 32) |
	        static_cast<uint64_t>(writeTime.dwLowDateTime);
	auto nanoseconds = std::chrono::duration_cast<std::chrono::nanoseconds>(
	        WindowsFiletimeDuration(timestamp));
	nanoseconds -= std::chrono::seconds(WINDOWS_EPOCH_OFFSET_SEC);

	return FilesystemTime(nanoseconds);
#else
	struct stat stats;
	int ret = stat(p.c_str(), &stats);
	if(ret != 0)
		bdrck::util::error::throwErrnoError();

	auto nanoseconds = std::chrono::duration_cast<std::chrono::nanoseconds>(
	        std::chrono::seconds(stats.st_mtim.tv_sec));
	nanoseconds += std::chrono::nanoseconds(stats.st_mtim.tv_nsec);

	return FilesystemTime(nanoseconds);
#endif
}

void lastWriteTime(std::string const &p, FilesystemTime const &t)
{
#ifdef _WIN32
	HANDLE file = CreateFile(p.c_str(), GENERIC_READ | GENERIC_WRITE,
	                         FILE_SHARE_READ, nullptr, OPEN_EXISTING,
	                         FILE_ATTRIBUTE_NORMAL, nullptr);
	if(file == INVALID_HANDLE_VALUE)
		throw std::runtime_error("Opening file handle failed.");
	bdrck::util::ScopeExit cleanup([&file]() { CloseHandle(file); });

	FILETIME writeTime;
	auto windowsTimestamp =
	        std::chrono::duration_cast<WindowsFiletimeDuration>(
	                t.time_since_epoch());
	windowsTimestamp += std::chrono::seconds(WINDOWS_EPOCH_OFFSET_SEC);
	uint64_t timestamp = static_cast<uint64_t>(windowsTimestamp.count());
	writeTime.dwHighDateTime = static_cast<DWORD>(timestamp >> 32);
	writeTime.dwLowDateTime =
	        static_cast<DWORD>(timestamp & 0x00000000FFFFFFFFULL);

	BOOL ret = SetFileTime(file, nullptr, nullptr, &writeTime);
	if(!ret)
		throw std::runtime_error("Setting file write time failed.");
#else
	auto duration = t.time_since_epoch();
	auto seconds =
	        std::chrono::duration_cast<std::chrono::seconds>(duration);
	auto nanoseconds =
	        std::chrono::duration_cast<std::chrono::nanoseconds>(duration) -
	        std::chrono::duration_cast<std::chrono::nanoseconds>(seconds);

	const struct timespec times[2] = {
	        {seconds.count(), nanoseconds.count()},
	        {seconds.count(), nanoseconds.count()}};

	int fd = open(p.c_str(), O_RDWR);
	if(fd == -1)
		bdrck::util::error::throwErrnoError();
	bdrck::util::ScopeExit cleanup([fd]() {
		int ret = close(fd);
		if(ret != 0)
			bdrck::util::error::throwErrnoError();
	});

	int ret = futimens(fd, times);
	if(ret != 0)
		bdrck::util::error::throwErrnoError();
#endif
}

void copyFile(std::string const &src, std::string const &dst)
{
	std::ofstream out(dst, std::ios_base::out | std::ios_base::binary |
	                               std::ios_base::trunc);
	std::ifstream in(src, std::ios_base::in | std::ios_base::binary);

	std::istreambuf_iterator<char> inBegin(in);
	std::istreambuf_iterator<char> inEnd;
	std::ostreambuf_iterator<char> outBegin(out);
	std::copy(inBegin, inEnd, outBegin);
}

std::string readEntireFile(std::string const &p)
{
	std::ostringstream out;
	std::ifstream in(p, std::ios_base::in | std::ios_base::binary);

	std::istreambuf_iterator<char> inBegin(in);
	std::istreambuf_iterator<char> inEnd;
	std::ostreambuf_iterator<char> outBegin(out);
	std::copy(inBegin, inEnd, outBegin);

	return out.str();
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
	boost::system::error_code ec;
	boost::filesystem::create_directory(p);
	if(ec)
		throw std::runtime_error(ec.message());
}

void removeDirectory(std::string const &p, bool recursive)
{
	if(recursive)
	{
		boost::system::error_code ec;
		boost::filesystem::remove_all(p, ec);
		if(ec)
			throw std::runtime_error(ec.message());
	}
	else
	{
		boost::system::error_code ec;
		boost::filesystem::remove(p, ec);
		if(ec)
			throw std::runtime_error(ec.message());
	}
}

void createPath(const std::string &p)
{
	std::vector<std::string> components =
	        bdrck::algorithm::string::split(normalizePath(p), '/');
	std::string currentPath = "/";

	for(const auto &component : components)
	{
		currentPath = combinePaths(currentPath, component);
		if(isDirectory(currentPath))
			continue;
		if(!exists(currentPath))
		{
			createDirectory(currentPath);
		}
		else
		{
			throw std::runtime_error("Create path failed because "
			                         "some path component already "
			                         "exists and is not a "
			                         "directory.");
		}
	}
}

void createSymlink(std::string const &target, std::string const &link)
{
	boost::system::error_code ec;
	boost::filesystem::create_symlink(target, link, ec);
	if(ec)
		throw std::runtime_error(ec.message());
}

std::string getCurrentExecutable()
{
#ifdef _WIN32
	CHAR path[MAX_PATH + 1];
	GetModuleFileName(nullptr, path, MAX_PATH + 1);
	return std::string(path);
#else
	char buffer[PATH_MAX];
	ssize_t length = ::readlink("/proc/self/exe", buffer, PATH_MAX);
	if(length == -1)
		bdrck::util::error::throwErrnoError();
	return std::string(&buffer[0], static_cast<std::size_t>(length));
#endif
}

std::string getCurrentDirectory()
{
	return dirname(getCurrentExecutable());
}

std::string getTemporaryDirectoryPath()
{
#ifdef _WIN32
	std::vector<TCHAR> buffer(MAX_PATH + 1);
	DWORD length =
	        GetTempPath(static_cast<DWORD>(buffer.size()), buffer.data());
	return resolvePath(bdrck::util::tstrToStdString(
	        buffer.data(), static_cast<std::size_t>(length)));
#else
	std::string path("/tmp");

	char const *tmpdir = std::getenv("TMPDIR");
	if(tmpdir != nullptr)
	{
		std::string tmpdirStr(tmpdir);
		if(isDirectory(tmpdirStr))
			path = tmpdirStr;
	}

	return path;
#endif
}

std::string
getConfigurationDirectoryPath(boost::optional<std::string> const &application)
{
#ifdef _WIN32
	PWSTR directory = nullptr;
	HRESULT ret = SHGetKnownFolderPath(FOLDERID_LocalAppData,
	                                   KF_FLAG_CREATE, nullptr, &directory);
	if(ret != S_OK)
	{
		throw std::runtime_error(
		        "Looking up application data directory failed.");
	}
	bdrck::util::ScopeExit cleanup(
	        [&directory]() { CoTaskMemFree(directory); });

	std::string path = bdrck::util::wstrToStdString(directory);
	if(!isDirectory(path))
	{
		throw std::runtime_error(
		        "Configuration directory is not a directory.");
	}

	if(!!application)
		path = combinePaths(path, *application);

	return normalizePath(path);
#else
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

	if(!!application)
		path = combinePaths(path, *application);

	return normalizePath(path);
#endif
}

boost::optional<std::string> which(std::string const &command,
                                   boost::optional<std::string> const &hint)
{
	char const *p = std::getenv("PATH");
	std::string path;
	if(p != nullptr)
		path.assign(p);

#ifdef _WIN32
	constexpr char PATH_DELIMITER = ';';
#else
	constexpr char PATH_DELIMITER = ':';
#endif
	std::vector<std::string> pathComponents =
	        bdrck::algorithm::string::split(path, PATH_DELIMITER);
	if(!!hint)
		pathComponents.insert(pathComponents.begin(), *hint);

	for(auto const &directory : pathComponents)
	{
		std::string commandPath = combinePaths(directory, command);

#ifdef _WIN32
		if(isExecutable(commandPath + ".exe"))
			return commandPath + ".exe";
#endif

		if(isExecutable(commandPath))
			return commandPath;
	}

	return boost::none;
}
}
}
