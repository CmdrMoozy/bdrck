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
}

namespace bdrck
{
namespace fs
{
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
}
}
