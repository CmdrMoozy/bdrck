#ifndef bdrck_fs_TemporaryStorage_HPP
#define bdrck_fs_TemporaryStorage_HPP

#include <string>

#include "bdrck/fs/Util.hpp"

namespace bdrck
{
namespace fs
{
enum class TemporaryStorageType
{
	FILE,
	DIRECTORY
};

class TemporaryStorage
{
public:
	TemporaryStorage(TemporaryStorageType t,
	                 std::string const &tempDir =
	                         bdrck::fs::getTemporaryDirectoryPath(),
	                 std::string const &prefix = "",
	                 std::string const &suffix = "");

	TemporaryStorage(TemporaryStorage const &) = delete;
	TemporaryStorage(TemporaryStorage &&) = default;
	TemporaryStorage &operator=(TemporaryStorage const &) = delete;
	TemporaryStorage &operator=(TemporaryStorage &&) = default;

	~TemporaryStorage();

	std::string getPath() const;

private:
	TemporaryStorageType type;
	std::string path;
};
}
}

#endif
