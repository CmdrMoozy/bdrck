#include "ExclusiveFileLock.hpp"

#include <stdexcept>

#include "bdrck/util/Error.hpp"

#ifdef _WIN32
#include <Windows.h>
#else
#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <sys/types.h>
#endif

namespace bdrck
{
namespace fs
{
namespace detail
{
#ifdef _WIN32
struct ExclusiveFileLockImpl
{
	HANDLE file;
	LARGE_INTEGER fileSize;
	OVERLAPPED overlap;

	ExclusiveFileLockImpl(std::string const &path)
	        : file(INVALID_HANDLE_VALUE), fileSize(), overlap()
	{
		ZeroMemory(&fileSize, sizeof(LARGE_INTEGER));
		ZeroMemory(&overlap, sizeof(OVERLAPPED));

		file = CreateFile(path.c_str(), GENERIC_READ | GENERIC_WRITE,
		                  FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
		                  OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
		                  nullptr);
		if(file == INVALID_HANDLE_VALUE)
			throw std::runtime_error("Failed to open file handle.");

		BOOL ret = GetFileSizeEx(file, &fileSize);
		if(!ret)
		{
			CloseHandle(file);
			throw std::runtime_error("Failed to get file size.");
		}

		ret = LockFileEx(file, LOCKFILE_EXCLUSIVE_LOCK |
		                               LOCKFILE_FAIL_IMMEDIATELY,
		                 0, fileSize.LowPart, fileSize.HighPart,
		                 &overlap);
		if(!ret)
		{
			CloseHandle(file);
			throw std::runtime_error(
			        "Acquiring exclusive file lock failed.");
		}
	}

	ExclusiveFileLockImpl(ExclusiveFileLockImpl const &) = delete;
	ExclusiveFileLockImpl(ExclusiveFileLockImpl &&) = default;
	ExclusiveFileLockImpl &
	operator=(ExclusiveFileLockImpl const &) = delete;
	ExclusiveFileLockImpl &operator=(ExclusiveFileLockImpl &&) = default;

	~ExclusiveFileLockImpl()
	{
		UnlockFileEx(file, 0, fileSize.LowPart, fileSize.HighPart,
		             &overlap);
		CloseHandle(file);
	}
};
#else
struct ExclusiveFileLockImpl
{
	int fd;

	ExclusiveFileLockImpl(std::string const &path) : fd(-1)
	{
		fd = open(path.c_str(), O_RDWR);
		if(fd == -1)
			bdrck::util::error::throwErrnoError();

		int ret = flock(fd, LOCK_EX);
		if(ret == -1)
		{
			close(fd);
			bdrck::util::error::throwErrnoError();
		}
	}

	ExclusiveFileLockImpl(ExclusiveFileLockImpl const &) = delete;
	ExclusiveFileLockImpl(ExclusiveFileLockImpl &&) = default;
	ExclusiveFileLockImpl &
	operator=(ExclusiveFileLockImpl const &) = delete;
	ExclusiveFileLockImpl &operator=(ExclusiveFileLockImpl &&) = default;

	~ExclusiveFileLockImpl()
	{
		flock(fd, LOCK_UN);
		close(fd);
	}
};
#endif
}

ExclusiveFileLock::ExclusiveFileLock(std::string const &path)
        : impl(std::make_unique<detail::ExclusiveFileLockImpl>(path))
{
}
}
}
