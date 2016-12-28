#include <catch/catch.hpp>

#include <chrono>
#include <fstream>

#include <boost/optional/optional.hpp>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/git/Commit.hpp"
#include "bdrck/git/Oid.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Signature.hpp"

namespace
{
bdrck::git::Signature getTestSignature()
{
	return bdrck::git::Signature("foo", "foo@bar.net",
	                             std::chrono::system_clock::now());
}
}

TEST_CASE("Test committing to new repository works", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	bdrck::git::Repository repository(directory.getPath());

	// Write some file to the repository to be committed.
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE,
	                                 directory.getPath());
	std::ofstream out(file.getPath());
	REQUIRE(out.is_open());
	out << "Foobar\n";
	out.close();

	boost::optional<bdrck::git::Oid> oid;
	REQUIRE_NOTHROW(oid = bdrck::git::commitAll(repository, "Test commit.",
	                                            getTestSignature(),
	                                            getTestSignature()));
	CHECK(!!oid);
}

TEST_CASE("Test empty commits work as intended", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	bdrck::git::Repository repository(directory.getPath());

	boost::optional<bdrck::git::Oid> oid;
	REQUIRE_NOTHROW(oid = bdrck::git::commitAll(repository, "Test commit.",
	                                            getTestSignature(),
	                                            getTestSignature()));
	CHECK(!oid);
}

TEST_CASE("Test commit then empty commit works as intended", "[Git]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	bdrck::git::Repository repository(directory.getPath());

	// Write some file to the repository to be committed.
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE,
	                                 directory.getPath());
	std::ofstream out(file.getPath());
	REQUIRE(out.is_open());
	out << "Foobar\n";
	out.close();

	boost::optional<bdrck::git::Oid> oidA;
	REQUIRE_NOTHROW(oidA = bdrck::git::commitAll(
	                        repository, "Test commit 1.",
	                        getTestSignature(), getTestSignature()));
	CHECK(!!oidA);

	boost::optional<bdrck::git::Oid> oidB;
	REQUIRE_NOTHROW(oidB = bdrck::git::commitAll(
	                        repository, "Test commit 2.",
	                        getTestSignature(), getTestSignature()));
	CHECK(!oidB);
}
