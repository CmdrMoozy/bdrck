#include <catch/catch.hpp>

#include <string>
#include <vector>

#include "bdrck/config/Util.hpp"

#include "TestConfiguration.pb.h"

TEST_CASE("Test nested field lookup", "[Configuration]")
{
	bdrck::test::messages::TestConfiguration message;
	auto descriptor =
	        bdrck::config::pathToDescriptor("sub.subsub.quux", message);
	REQUIRE(descriptor.first != nullptr);
	REQUIRE(descriptor.second != nullptr);
}

TEST_CASE("Test set / get via string round tripping", "[Configuration]")
{
	typedef struct
	{
		std::string path;
		std::string value;
	} TestCase;
	static const std::vector<TestCase> TEST_CASES{
	        {"a", "-13478"}, {"b", "-23478"},  {"c", "-13478"},
	        {"d", "-137"},   {"e", "-123478"}, {"f", "-1278"},
	        {"g", "1377"},   {"h", "12573"},   {"i", "1235"},
	        {"j", "1346"},   {"m", "false"},   {"m", "true"},
	        {"o", "foobar"}};

	bdrck::test::messages::RoundTripTest message;
	for(auto const &testCase : TEST_CASES)
	{
		bdrck::config::setFieldFromString(testCase.path, message,
		                                  testCase.value);
		CHECK(bdrck::config::getFieldAsString(testCase.path, message) ==
		      testCase.value);
	}
}
