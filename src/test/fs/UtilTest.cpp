#include <catch/catch.hpp>

#include <fstream>
#include <string>

#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/fs/Util.hpp"

TEST_CASE("Test combinePath", "[Util]")
{
	using TestCase = struct
	{
		std::string a;
		std::string b;
		std::string expected;
	};

	static const std::vector<TestCase> TEST_CASES = {
	        {"", "", ""},
	        {"", "/", "/"},
	        {"/", "", "/"},
	        {"foo/bar", "baz/quux", "foo/bar/baz/quux"},
	        {"/foo/bar", "baz/quux", "/foo/bar/baz/quux"}};

	for(auto const &testCase : TEST_CASES)
	{
		CHECK(bdrck::fs::combinePaths(testCase.a, testCase.b) ==
		      testCase.expected);
	}
}

TEST_CASE("Test createFile", "[Util]")
{
	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	std::string filePath =
	        bdrck::fs::combinePaths(directory.getPath(), "testfile");
	CHECK(!bdrck::fs::isFile(filePath));
	bdrck::fs::createFile(filePath);
	CHECK(bdrck::fs::isFile(filePath));
}

TEST_CASE("Test copyFile", "[Util]")
{
	constexpr char const *TEST_CONTENTS = "this is a test file\n";

	bdrck::fs::TemporaryStorage directory(
	        bdrck::fs::TemporaryStorageType::DIRECTORY);
	std::string aPath = bdrck::fs::combinePaths(directory.getPath(), "a");
	std::string bPath = bdrck::fs::combinePaths(directory.getPath(), "b");

	std::ofstream aOut(aPath, std::ios_base::out | std::ios_base::binary |
	                                  std::ios_base::trunc);
	aOut << TEST_CONTENTS;
	aOut.close();

	bdrck::fs::copyFile(aPath, bPath);
	CHECK(bdrck::fs::readEntireFile(bPath) == std::string(TEST_CONTENTS));
}
