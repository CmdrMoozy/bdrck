#include <catch/catch.hpp>

#include <string>

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
