#include "TemporaryStorage.hpp"

#include <cassert>
#include <sstream>
#include <stdexcept>

#include "bdrck/fs/Util.hpp"
#include "bdrck/util/UUID.hpp"

namespace
{
std::string getTemporaryPath(std::string const &tempDir,
                             std::string const &prefix,
                             std::string const &suffix)
{
	std::ostringstream oss;
	oss << prefix << bdrck::util::generateUUID() << suffix;
	return bdrck::fs::normalizePath(
	        bdrck::fs::combinePaths(tempDir, oss.str()));
}
}

namespace bdrck
{
namespace fs
{
TemporaryStorage::TemporaryStorage(TemporaryStorageType t,
                                   std::string const &tempDir,
                                   std::string const &prefix,
                                   std::string const &suffix)
        : type(t), path(getTemporaryPath(tempDir, prefix, suffix))
{
	while(exists(path))
		path = getTemporaryPath(tempDir, prefix, suffix);

	switch(type)
	{
	case TemporaryStorageType::FILE:
		createFile(path);
		if(!isFile(path))
		{
			throw std::runtime_error(
			        "Creating temporary file failed.");
		}
		break;

	case TemporaryStorageType::DIRECTORY:
		createDirectory(path);
		if(!isDirectory(path))
		{
			throw std::runtime_error(
			        "Creating temporary directory failed.");
		}
		break;
	}
}

TemporaryStorage::~TemporaryStorage()
{
	switch(type)
	{
	case TemporaryStorageType::FILE:
		removeFile(path);
		break;

	case TemporaryStorageType::DIRECTORY:
		removeDirectory(path);
		break;
	}
}

std::string TemporaryStorage::getPath() const
{
	return path;
}
}
}
