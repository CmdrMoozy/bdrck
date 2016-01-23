#include <catch/catch.hpp>

#include <cmath>
#include <sstream>
#include <string>

#include <boost/variant/get.hpp>

#include "bdrck/json/parse.hpp"

TEST_CASE("Test parsing empty files", "[parse]")
{
	std::string input("");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	CHECK(!result);
}

TEST_CASE("Test single null value", "[parse]")
{
	std::string input("null\n");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);
	CHECK(boost::get<bdrck::json::NullType>(*result) ==
	      bdrck::json::NullType());
}

TEST_CASE("Test single boolean value", "[parse]")
{
	std::string input("true");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);
	CHECK(boost::get<bdrck::json::BooleanType>(*result) == true);
}

TEST_CASE("Test single integer value", "[parse]")
{
	std::string input("12345");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);
	CHECK(boost::get<bdrck::json::IntegerType>(*result) == 12345);
}

TEST_CASE("Test single double value", "[parse]")
{
	std::string input("123.456");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);
	CHECK(std::abs(boost::get<bdrck::json::DoubleType>(*result) - 123.456) <
	      0.001);
}

TEST_CASE("Test single string value", "[parse]")
{
	std::string input("\"test value\"");
	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);
	CHECK(bdrck::json::toString(boost::get<bdrck::json::StringType>(
	              *result)) == "test value");
}

TEST_CASE("Test complex data structure parsing", "[parse]")
{
	// clang-format off
	std::string input(
		"{"
			"\"foo\": ["
				"{"
					"\"baz\": \"quux\""
				"},"
				"12345,"
				"\"foobar\""
			"],"
			"\"bar\": {"
				"\"foo\": ["
					"null,"
					"true,"
					"123.456"
				"]"
			"}"
		"}"
	);
	// clang-format on

	std::istringstream iss(input);
	auto result = bdrck::json::parseAll(iss);
	REQUIRE(!!result);

	// Test the outer map.
	auto const &map = boost::get<bdrck::json::MapType>(*result);

	// Test the array value from the outer map.
	auto subArrayIt = map.find(bdrck::json::fromString("foo"));
	REQUIRE(subArrayIt != map.end());
	auto const &subArray =
	        boost::get<bdrck::json::ArrayType>(subArrayIt->second);
	REQUIRE(subArray.size() == 3);

	// Test the map member of the array member of the outer map.
	auto subSubMap = boost::get<bdrck::json::MapType>(subArray[0]);
	auto subSubMapValueIt = subSubMap.find(bdrck::json::fromString("baz"));
	CHECK(bdrck::json::toString(boost::get<bdrck::json::StringType>(
	              subSubMapValueIt->second)) == "quux");

	// Test the integer member of the array member of the outer map.
	auto subSubInteger = boost::get<bdrck::json::IntegerType>(subArray[1]);
	CHECK(subSubInteger == 12345);

	// Test the string member of the array member of the outer map.
	auto subSubString = boost::get<bdrck::json::StringType>(subArray[2]);
	CHECK(bdrck::json::toString(subSubString) == "foobar");

	// Test the map value from the outer map.
	auto subMapIt = map.find(bdrck::json::fromString("bar"));
	REQUIRE(subMapIt != map.end());
	auto const &subMap = boost::get<bdrck::json::MapType>(subMapIt->second);
	REQUIRE(subMap.size() == 1);

	// Test the array member of the map member of the outer map.
	auto subSubArrayIt = subMap.find(bdrck::json::fromString("foo"));
	REQUIRE(subSubArrayIt != subMap.end());
	auto const &subSubArray =
	        boost::get<bdrck::json::ArrayType>(subSubArrayIt->second);
	REQUIRE(subSubArray.size() == 3);

	// Test the null member of the sub sub array.
	auto subSubSubNull = boost::get<bdrck::json::NullType>(subSubArray[0]);

	// Test the boolean member of the sub sub array.
	auto subSubSubBool =
	        boost::get<bdrck::json::BooleanType>(subSubArray[1]);
	CHECK(subSubSubBool == true);

	// Test the double member of the sub sub array.
	auto subSubSubDouble =
	        boost::get<bdrck::json::DoubleType>(subSubArray[2]);
	CHECK(std::abs(subSubSubDouble - 123.456) < 0.001);
}
