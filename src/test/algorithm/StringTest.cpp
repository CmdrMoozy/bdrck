#include <catch/catch.hpp>

#include <string>
#include <utility>
#include <vector>

#include "bdrck/algorithm/String.hpp"

TEST_CASE("Test string lowercasing algorithm", "[String]")
{
	const std::vector<std::pair<std::string, std::string>> TEST_CASES{
	        {"", ""},
	        {" 1234567890 !@#$%^&*() -= \\/+_",
	         " 1234567890 !@#$%^&*() -= \\/+_"},
	        {"abcdefghijklmnopqrstuvwxyz", "abcdefghijklmnopqrstuvwxyz"},
	        {"ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmnopqrstuvwxyz"},
	        {"17#@&$*dAcJfHssdkFKdjsS(9", "17#@&$*dacjfhssdkfkdjss(9"},
	        {"   \t   ", "   \t   "}};

	for(auto const &test : TEST_CASES)
	{
		auto output = bdrck::algorithm::string::toLower(test.first);
		CHECK(test.second == output);
	}
}

TEST_CASE("Test string split algorithm", "[String]")
{
	const char TEST_DELIMITER = ',';
	const std::vector<std::pair<std::string, std::vector<std::string>>>
	        TEST_DATA = {{"", {}},
	                     {",,,,,,,,", {}},
	                     {"foobar", {"foobar"}},
	                     {",,foobar", {"foobar"}},
	                     {"foobar,,", {"foobar"}},
	                     {",,,,foobar,,,,", {"foobar"}},
	                     {",,,,foo,,,,bar,,,,", {"foo", "bar"}},
	                     {"f,o,o,b,a,r", {"f", "o", "o", "b", "a", "r"}}};

	for(auto const &test : TEST_DATA)
	{
		auto output = bdrck::algorithm::string::split(test.first,
		                                              TEST_DELIMITER);
		CHECK(test.second == output);
	}
}

namespace
{
struct JoinTestCase
{
	std::vector<std::string> input;
	std::string delimiter;
	std::string expected;

	JoinTestCase(std::vector<std::string> const &i, std::string const &d,
	             std::string const &e)
	        : input(i), delimiter(d), expected(e)
	{
	}
};
}

TEST_CASE("Test string join algorithm", "[String]")
{
	const std::vector<JoinTestCase> TEST_CASES{
	        {{"foo", "bar", "baz"}, " ", "foo bar baz"},
	        {{}, "foobar", ""},
	        {{"", "", ""}, ",", ",,"},
	        {{"foo", "bar", "baz"}, "", "foobarbaz"}};

	for(auto const &test : TEST_CASES)
	{
		std::string output = bdrck::algorithm::string::join(
		        test.input.begin(), test.input.end(), test.delimiter);
		CHECK(test.expected == output);
	}
}

TEST_CASE("Test string left trim algorithm", "[String]")
{
	const std::vector<std::pair<std::string, std::string>> TEST_CASES{
	        {"", ""},
	        {"foobar", "foobar"},
	        {"foobar\t\n ", "foobar\t\n "},
	        {"\n\n\nfoobar", "foobar"},
	        {"\t \t \n ", ""},
	        {"\t \t \n foobar", "foobar"},
	        {"foobar \t\n foobar", "foobar \t\n foobar"}};

	for(auto const &test : TEST_CASES)
	{
		std::string result(test.first);
		bdrck::algorithm::string::leftTrim(result);
		CHECK(test.second == result);
	}
}

TEST_CASE("Test string right trim algorithm", "[String]")
{
	const std::vector<std::pair<std::string, std::string>> TEST_CASES{
	        {"", ""},
	        {"foobar", "foobar"},
	        {"foobar\t\n ", "foobar"},
	        {"foobar\n\n\n", "foobar"},
	        {"\n\n\nfoobar", "\n\n\nfoobar"},
	        {"\t \t \n ", ""},
	        {"foobar\t \t \n ", "foobar"},
	        {"foobar \t\n foobar", "foobar \t\n foobar"}};

	for(auto const &test : TEST_CASES)
	{
		std::string result(test.first);
		bdrck::algorithm::string::rightTrim(result);
		CHECK(test.second == result);
	}
}

TEST_CASE("Test string trim algorithm", "[String]")
{
	const std::vector<std::pair<std::string, std::string>> TEST_CASES{
	        {"", ""},
	        {"foobar", "foobar"},
	        {"foobar\t\n ", "foobar"},
	        {"foobar\n\n\n", "foobar"},
	        {"\n\n\nfoobar", "foobar"},
	        {"\t \t \n ", ""},
	        {"foobar\t \t \n ", "foobar"},
	        {"foobar \t\n foobar", "foobar \t\n foobar"}};

	for(auto const &test : TEST_CASES)
	{
		std::string result(test.first);
		bdrck::algorithm::string::trim(result);
		CHECK(test.second == result);
	}
}

namespace
{
struct RemoveRepeatedCharacterTestCase
{
	std::string input;
	char character;
	std::string expected;

	RemoveRepeatedCharacterTestCase(std::string const &i, char c,
	                                std::string const &e)
	        : input(i), character(c), expected(e)
	{
	}
};
}

TEST_CASE("Test repeated character removal", "[String]")
{
	const std::vector<RemoveRepeatedCharacterTestCase> TEST_CASES{
	        {"", ' ', ""},
	        {"abcdefghijklmnop", 'g', "abcdefghijklmnop"},
	        {"/foo/bar//baz/test/foobar//", '/',
	         "/foo/bar/baz/test/foobar/"},
	        {"//////////", '/', "/"},
	        {"/", '/', "/"}};

	for(auto const &test : TEST_CASES)
	{
		std::string result(test.input);
		bdrck::algorithm::string::removeRepeatedCharacters(
		        result, test.character);
		CHECK(test.expected == result);
	}
}
