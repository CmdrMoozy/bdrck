#include "TemporaryStorage.hpp"

#include <cassert>
#include <sstream>
#include <stdexcept>

#include "bdrck/fs/Util.hpp"
#include "bdrck/util/UUID.hpp"

namespace
{
std::string getTemporaryPath()
{
	std::ostringstream oss;
	oss << "bdrck-" << bdrck::util::generateUUID() << ".tmp";
	return bdrck::fs::combinePaths(bdrck::fs::getTemporaryDirectoryPath(),
	                               oss.str());
}
}

namespace bdrck
{
namespace fs
{
TemporaryStorage::TemporaryStorage(TemporaryStorageType t)
        : type(t), path(getTemporaryPath())
{
	while(exists(path))
		path = getTemporaryPath();

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
