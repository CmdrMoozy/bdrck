#ifndef bdrck_fs_TemporaryStorage_HPP
#define bdrck_fs_TemporaryStorage_HPP

#include <string>

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
	explicit TemporaryStorage(TemporaryStorageType t);

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
