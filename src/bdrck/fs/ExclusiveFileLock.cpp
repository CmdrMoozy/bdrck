#include "ExclusiveFileLock.hpp"

#include "bdrck/util/Error.hpp"

#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <sys/types.h>

namespace bdrck
{
namespace fs
{
namespace detail
{
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
}

ExclusiveFileLock::ExclusiveFileLock(std::string const &path)
        : impl(std::make_unique<detail::ExclusiveFileLockImpl>(path))
{
}
}
}
