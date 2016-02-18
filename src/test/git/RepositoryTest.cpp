#include <catch/catch.hpp>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/fs/Util.hpp"
#include "bdrck/git/Repository.hpp"

TEST_CASE("Test Git repository work directory path retrieval", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	REQUIRE(bdrck::fs::isDirectory(directory.getPath()));
	bdrck::git::Repository repository(directory.getPath());
	CHECK(bdrck::fs::normalizePath(directory.getPath()) ==
	      bdrck::fs::normalizePath(repository.getWorkDirectoryPath()));
}
