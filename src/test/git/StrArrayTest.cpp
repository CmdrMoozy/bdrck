#include <catch/catch.hpp>

#include <cstring>
#include <string>
#include <vector>

#include <git2.h>

#include "bdrck/git/StrArray.hpp"

TEST_CASE("Test StrArray initialization", "[Git]")
{
	const std::vector<std::string> TEST_STRINGS = {"foo", "bar", "baz",
	                                               "quux"};

	bdrck::git::StrArray container(TEST_STRINGS.begin(),
	                               TEST_STRINGS.end());
	git_strarray const &strarray = container.get();

	for(std::size_t i = 0; i < strarray.count; ++i)
	{
		CHECK(std::strlen(strarray.strings[i]) ==
		      TEST_STRINGS[i].length());
		CHECK(std::strcmp(strarray.strings[i],
		                  TEST_STRINGS[i].c_str()) == 0);
	}
}
