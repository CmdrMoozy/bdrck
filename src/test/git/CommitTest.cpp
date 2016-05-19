#include <catch/catch.hpp>

#include <chrono>
#include <fstream>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/git/Commit.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Signature.hpp"

TEST_CASE("Test committing to new repository works", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	bdrck::git::Repository repository(directory.getPath());

	bdrck::git::Signature signature("foo", "foo@bar.net",
	                                std::chrono::system_clock::now());

	// Write some file to the repository and commit it.
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE,
	                                 directory.getPath());
	std::ofstream out(file.getPath());
	REQUIRE(out.is_open());
	out << "Foobar\n";
	out.close();
	REQUIRE_NOTHROW(bdrck::git::commitAll(repository, "Test commit.",
	                                      signature, signature));
}
