#include <catch/catch.hpp>

#include <algorithm>
#include <string>
#include <vector>

#include "bdrck/fs/Iterator.hpp"
#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/fs/Util.hpp"

namespace
{
struct TestContext
{
	bdrck::fs::TemporaryStorage directoryA;
	bdrck::fs::TemporaryStorage directoryB;

	std::vector<std::string> paths;
	std::vector<std::string> pathsWithoutSymlinks;

	TestContext()
	        : directoryA(bdrck::fs::TemporaryStorageType::DIRECTORY),
	          directoryB(bdrck::fs::TemporaryStorageType::DIRECTORY),
	          paths(),
	          pathsWithoutSymlinks()
	{
		const std::string subdirPath =
		        bdrck::fs::combinePaths(directoryA.getPath(), "subdir");
		const std::string fileAPath =
		        bdrck::fs::combinePaths(directoryA.getPath(), "fileA");
		const std::string fileBPath =
		        bdrck::fs::combinePaths(subdirPath, "fileB");
		const std::string fileCPath =
		        bdrck::fs::combinePaths(directoryB.getPath(), "fileC");
		const std::string symlinkAPath = bdrck::fs::combinePaths(
		        directoryA.getPath(), "symlinkA");
		const std::string symlinkBPath =
		        bdrck::fs::combinePaths(subdirPath, "symlinkB");
		const std::string symlinkCPath = bdrck::fs::combinePaths(
		        directoryA.getPath(), "symlinkC");
		const std::string fileCSymlinkPath =
		        bdrck::fs::combinePaths(symlinkAPath, "fileC");

		bdrck::fs::createDirectory(subdirPath);
		bdrck::fs::createFile(fileAPath);
		bdrck::fs::createFile(fileBPath);
		bdrck::fs::createFile(fileCPath);
		bdrck::fs::createSymlink(directoryB.getPath(), symlinkAPath);
		bdrck::fs::createSymlink(fileAPath, symlinkBPath);
		bdrck::fs::createSymlink(
		        bdrck::fs::combinePaths(directoryA.getPath(),
		                                "NON_EXISTENT_PATH"),
		        symlinkCPath);

		paths = {directoryA.getPath(),
		         subdirPath,
		         fileAPath,
		         fileBPath,
		         fileCSymlinkPath,
		         symlinkAPath,
		         symlinkBPath,
		         symlinkCPath};

		pathsWithoutSymlinks = {directoryA.getPath(),
		                        subdirPath,
		                        fileAPath,
		                        fileBPath,
		                        symlinkAPath,
		                        symlinkBPath,
		                        symlinkCPath};

		std::sort(paths.begin(), paths.end());
		std::sort(pathsWithoutSymlinks.begin(),
		          pathsWithoutSymlinks.end());
	}
};
}

TEST_CASE("Test filesystem iteration not following symlinks", "[Iterator]")
{
	TestContext context;
	bdrck::fs::Iterator it(context.directoryA.getPath(), false, false);
	bdrck::fs::Iterator end;
	std::vector<std::string> paths;
	for(; it != end; ++it)
		paths.emplace_back(*it);
	std::sort(paths.begin(), paths.end());

	REQUIRE(paths == context.pathsWithoutSymlinks);
}

TEST_CASE("Test filesystem iteration following symlinks", "[Iterator]")
{
	TestContext context;
	bdrck::fs::Iterator it(context.directoryA.getPath(), true, false);
	bdrck::fs::Iterator end;
	std::vector<std::string> paths;
	for(; it != end; ++it)
		paths.emplace_back(*it);
	std::sort(paths.begin(), paths.end());

	REQUIRE(paths == context.paths);
}
