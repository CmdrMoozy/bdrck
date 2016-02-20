#include <catch/catch.hpp>

#include <boost/optional/optional.hpp>

#include "bdrck/params/Argument.hpp"
#include "bdrck/params/Command.hpp"

TEST_CASE("Test command construction with valid defaulted arguments",
          "[Parameters]")
{
	boost::optional<bdrck::params::Command> command;
	REQUIRE_NOTHROW(command.emplace(
	        "test", "A test command.", bdrck::params::CommandFunction(),
	        std::initializer_list<bdrck::params::Option>({}),
	        std::vector<bdrck::params::Argument>(
	                {bdrck::params::Argument("foo", "foo"),
	                 bdrck::params::Argument("bar", "bar", "foobar"),
	                 bdrck::params::Argument("baz", "baz", "barbaz")}),
	        false));
}

TEST_CASE("Test command construction with invalid defaulted arguments",
          "[Parameters]")
{
	boost::optional<bdrck::params::Command> command;
	REQUIRE_THROWS(command.emplace(
	        "test", "A test command.", bdrck::params::CommandFunction(),
	        std::initializer_list<bdrck::params::Option>({}),
	        std::vector<bdrck::params::Argument>(
	                {bdrck::params::Argument("foo", "foo"),
	                 bdrck::params::Argument("bar", "bar", "foobar"),
	                 bdrck::params::Argument("baz", "baz")}),
	        false));
}
