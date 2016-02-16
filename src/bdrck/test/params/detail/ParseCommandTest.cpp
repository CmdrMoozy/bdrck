#include <catch/catch.hpp>

#include <set>

#include "bdrck/params/Command.hpp"
#include "bdrck/params/ProgramParameters.hpp"
#include "bdrck/params/detail/parseCommand.hpp"

TEST_CASE("Test invalid command", "[Parameters]")
{
	std::set<bdrck::params::Command> commands;
	commands.emplace("foo", "foo", bdrck::params::CommandFunction());
	commands.emplace("bar", "bar", bdrck::params::CommandFunction());
	commands.emplace("baz", "baz", bdrck::params::CommandFunction());

	bdrck::params::ProgramParameters parameters{"biff", "foo", "bar",
	                                            "baz"};
	REQUIRE(4 == parameters.parameters.size());
	CHECK(commands.cend() ==
	      bdrck::params::detail::parseCommand(parameters, commands));
	CHECK(4 == parameters.parameters.size());
}

TEST_CASE("Test command with no arguments", "[Parameters]")
{
	std::set<bdrck::params::Command> commands;
	commands.emplace("foo", "foo", bdrck::params::CommandFunction());
	const auto barIt =
	        commands.emplace("bar", "bar", bdrck::params::CommandFunction())
	                .first;
	commands.emplace("baz", "baz", bdrck::params::CommandFunction());

	bdrck::params::ProgramParameters parameters{"bar"};
	REQUIRE(1 == parameters.parameters.size());
	CHECK(barIt ==
	      bdrck::params::detail::parseCommand(parameters, commands));
	CHECK(0 == parameters.parameters.size());
}

TEST_CASE("Test command with arguments", "[Parameters]")
{
	std::set<bdrck::params::Command> commands;
	commands.emplace("foo", "foo", bdrck::params::CommandFunction());
	commands.emplace("bar", "bar", bdrck::params::CommandFunction());
	const auto bazIt =
	        commands.emplace("baz", "baz", bdrck::params::CommandFunction())
	                .first;

	bdrck::params::ProgramParameters parameters{"baz", "foo", "bar", "baz"};
	REQUIRE(4 == parameters.parameters.size());
	CHECK(bazIt ==
	      bdrck::params::detail::parseCommand(parameters, commands));
	CHECK(3 == parameters.parameters.size());
}
