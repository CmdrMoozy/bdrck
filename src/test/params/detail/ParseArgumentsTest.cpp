#include <catch/catch.hpp>

#include "bdrck/params/Argument.hpp"
#include "bdrck/params/Command.hpp"
#include "bdrck/params/ProgramParameters.hpp"
#include "bdrck/params/detail/parseArguments.hpp"

namespace
{
bool valuesArePresent(bdrck::params::ArgumentsMap const &arguments,
                      std::string const &name,
                      std::vector<std::string> const &values)
{
	auto it = arguments.find(name);
	if(it == arguments.end())
		return false;
	return it->second == values;
}
}

TEST_CASE("Test normal argument parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"oof", "rab", "zab"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", ""),
	         bdrck::params::Argument("bar", ""),
	         bdrck::params::Argument("baz", "")},
	        false);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_NOTHROW(arguments = bdrck::params::detail::parseArguments(
	                        parameters, command));

	CHECK(arguments.size() == 3);
	CHECK(valuesArePresent(arguments, "foo", {"oof"}));
	CHECK(valuesArePresent(arguments, "bar", {"rab"}));
	CHECK(valuesArePresent(arguments, "baz", {"zab"}));
	CHECK(parameters.parameters.empty());
}

TEST_CASE("Test multiple default values", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"a", "b", "c"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", ""),
	         bdrck::params::Argument("bar", ""),
	         bdrck::params::Argument("baz", ""),
	         bdrck::params::Argument("oof", "", "A"),
	         bdrck::params::Argument("rab", "", "B"),
	         bdrck::params::Argument("zab", "", "C")},
	        false);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_NOTHROW(arguments = bdrck::params::detail::parseArguments(
	                        parameters, command));

	CHECK(arguments.size() == 6);
	CHECK(valuesArePresent(arguments, "foo", {"a"}));
	CHECK(valuesArePresent(arguments, "bar", {"b"}));
	CHECK(valuesArePresent(arguments, "baz", {"c"}));
	CHECK(valuesArePresent(arguments, "oof", {"A"}));
	CHECK(valuesArePresent(arguments, "rab", {"B"}));
	CHECK(valuesArePresent(arguments, "zab", {"C"}));
	CHECK(parameters.parameters.empty());
}

TEST_CASE("Test variadic last argument with default value", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"a"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", ""),
	         bdrck::params::Argument("bar", "", "foobar")},
	        true);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_NOTHROW(arguments = bdrck::params::detail::parseArguments(
	                        parameters, command));

	CHECK(arguments.size() == 2);
	CHECK(valuesArePresent(arguments, "foo", {"a"}));
	CHECK(valuesArePresent(arguments, "bar", {"foobar"}));
	CHECK(parameters.parameters.empty());
}

TEST_CASE("Test variadic last argument with single value", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"a", "b"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", ""),
	         bdrck::params::Argument("bar", "", "foobar")},
	        true);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_NOTHROW(arguments = bdrck::params::detail::parseArguments(
	                        parameters, command));

	CHECK(arguments.size() == 2);
	CHECK(valuesArePresent(arguments, "foo", {"a"}));
	CHECK(valuesArePresent(arguments, "bar", {"b"}));
	CHECK(parameters.parameters.empty());
}

TEST_CASE("Test variadic last argument with multiple values", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"a", "b", "c", "d"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", ""),
	         bdrck::params::Argument("bar", "", "foobar")},
	        true);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_NOTHROW(arguments = bdrck::params::detail::parseArguments(
	                        parameters, command));

	CHECK(arguments.size() == 2);
	CHECK(valuesArePresent(arguments, "foo", {"a"}));
	CHECK(valuesArePresent(arguments, "bar", {"b", "c", "d"}));
	CHECK(parameters.parameters.empty());
}

TEST_CASE("Test extra program parameters", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters({"bar", "baz"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(), {},
	        {bdrck::params::Argument("foo", "")}, false);

	bdrck::params::ArgumentsMap arguments;
	REQUIRE_THROWS(arguments = bdrck::params::detail::parseArguments(
	                       parameters, command));
}
