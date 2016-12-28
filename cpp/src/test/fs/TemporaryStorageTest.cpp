#include <catch/catch.hpp>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/fs/Util.hpp"

TEST_CASE("Test temporary file behavior", "[TemporaryStorage]")
{
	std::string path;

	{
		bdrck::fs::TemporaryStorage file(
		        bdrck::fs::TemporaryStorageType::FILE);
		path = file.getPath();
		CHECK(bdrck::fs::exists(path));
		CHECK(bdrck::fs::isFile(path));
	}

	CHECK(!bdrck::fs::isFile(path));
	CHECK(!bdrck::fs::exists(path));
}

TEST_CASE("Test temporary directory behavior", "[TemporaryStorage]")
{
	std::string path;

	{
		bdrck::fs::TemporaryStorage directory(
		        bdrck::fs::TemporaryStorageType::DIRECTORY);
		path = directory.getPath();
		CHECK(bdrck::fs::exists(path));
		CHECK(bdrck::fs::isDirectory(path));

		// Add some random files and directories to the directory, to
		// make sure the removal still works if it is non-empty.

		std::string afile = bdrck::fs::combinePaths(path, "a.txt");
		std::string bdir = bdrck::fs::combinePaths(path, "b");
		std::string bdirafile = bdrck::fs::combinePaths(bdir, "a.txt");
		std::string bdircdir = bdrck::fs::combinePaths(bdir, "c");
		std::string bdircdirafile =
		        bdrck::fs::combinePaths(bdircdir, "a.txt");

		CHECK_NOTHROW(bdrck::fs::createFile(afile));
		CHECK_NOTHROW(bdrck::fs::createDirectory(bdir));
		CHECK_NOTHROW(bdrck::fs::createFile(bdirafile));
		CHECK_NOTHROW(bdrck::fs::createDirectory(bdircdir));
		CHECK_NOTHROW(bdrck::fs::createFile(bdircdirafile));
	}

	CHECK(!bdrck::fs::isDirectory(path));
	CHECK(!bdrck::fs::exists(path));
}
