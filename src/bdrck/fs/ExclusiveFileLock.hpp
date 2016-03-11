#ifndef bdrck_fs_ExclusiveFileLock_HPP
#define bdrck_fs_ExclusiveFileLock_HPP

#include <memory>
#include <string>

namespace bdrck
{
namespace fs
{
namespace detail
{
struct ExclusiveFileLockImpl;
}

class ExclusiveFileLock
{
public:
	ExclusiveFileLock(std::string const &path);

	ExclusiveFileLock(ExclusiveFileLock const &) = delete;
	ExclusiveFileLock(ExclusiveFileLock &&) = default;
	ExclusiveFileLock &operator=(ExclusiveFileLock const &) = delete;
	ExclusiveFileLock &operator=(ExclusiveFileLock &&) = default;

	~ExclusiveFileLock();

private:
	std::unique_ptr<detail::ExclusiveFileLockImpl> impl;
};
}
}

#endif
