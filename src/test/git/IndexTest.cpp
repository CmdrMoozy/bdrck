#include <catch/catch.hpp>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/fs/Util.hpp"
#include "bdrck/git/Index.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/StrArray.hpp"

TEST_CASE("Test adding files to index", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	bdrck::git::Repository repository(directory.getPath());
	bdrck::git::Index index(repository);
	REQUIRE(index.getEntryCount() == 0);

	// Write two files into the directory.
	bdrck::fs::TemporaryStorage file1(bdrck::fs::TemporaryStorageType::FILE,
	                                  directory.getPath());
	bdrck::fs::TemporaryStorage file2(bdrck::fs::TemporaryStorageType::FILE,
	                                  directory.getPath());

	// The index should still be empty.
	REQUIRE(index.getEntryCount() == 0);

	// Add one file to the index.
	index.addAll({bdrck::fs::basename(file1.getPath())});

	// The index should now have one entry.
	REQUIRE(index.getEntryCount() == 1);

	// Add everything else to the index.
	index.addAll({"."});

	// The index should now have two entries.
	REQUIRE(index.getEntryCount() == 2);

	// If we instantiate a new index, it should also report two entries.
	bdrck::git::Index newIndex(repository);
	REQUIRE(index.getEntryCount() == 2);
}
