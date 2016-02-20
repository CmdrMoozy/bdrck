#include <catch/catch.hpp>

#include <tuple>

#include <boost/optional/optional.hpp>

#include "bdrck/params/Command.hpp"
#include "bdrck/params/Option.hpp"
#include "bdrck/params/ProgramParameters.hpp"
#include "bdrck/params/detail/parseOptions.hpp"

namespace
{
const bdrck::params::Command TEST_COMMAND(
        "test", "A command for testing purposes.",
        bdrck::params::CommandFunction(),
        {bdrck::params::Option::flag("flaga", "", 'a'),
         bdrck::params::Option::required("optiona", "", 'A'),
         bdrck::params::Option::flag("flagb", "", 'b'),
         bdrck::params::Option::required("optionb", "", 'B', "bdefault"),
         bdrck::params::Option::flag("flagc", "", 'c'),
         bdrck::params::Option::required("optionc", "", 'C')},
        {}, false);

bool optionValueCorrect(std::string const &name,
                        std::string const &expectedValue,
                        std::tuple<bdrck::params::OptionsMap,
                                   bdrck::params::FlagsMap> const &parsed)
{
	auto it = std::get<0>(parsed).find(name);
	if(it == std::get<0>(parsed).end())
		return false;
	return it->second == expectedValue;
}

bool flagValueCorrect(std::string const &name, bool expectedValue,
                      std::tuple<bdrck::params::OptionsMap,
                                 bdrck::params::FlagsMap> const &parsed)
{
	auto it = std::get<1>(parsed).find(name);
	if(it == std::get<1>(parsed).end())
		return false;
	return it->second == expectedValue;
}
}

TEST_CASE("Test mixed name option parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"--flaga", "--optiona", "foobar", "--flagb", "-B", "barbaz",
	         "-c", "--optionc", "foobaz"});

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_NOTHROW(parsed = bdrck::params::detail::parseOptions(
	                        parameters, TEST_COMMAND));
	CHECK(parameters.parameters.size() == 0);

	CHECK(flagValueCorrect("flaga", true, parsed));
	CHECK(optionValueCorrect("optiona", "foobar", parsed));
	CHECK(flagValueCorrect("flagb", true, parsed));
	CHECK(optionValueCorrect("optionb", "barbaz", parsed));
	CHECK(flagValueCorrect("flagc", true, parsed));
	CHECK(optionValueCorrect("optionc", "foobaz", parsed));
}

TEST_CASE("Test missing options after parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"--flaga", "-b", "--optiona", "foobar"});

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_THROWS(parsed = bdrck::params::detail::parseOptions(
	                       parameters, TEST_COMMAND));
	CHECK(parameters.parameters.size() == 0);
}

TEST_CASE("Test defaulted option values", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"--flaga", "-c", "--optiona", "foobar", "-C", "barbaz"});

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_NOTHROW(parsed = bdrck::params::detail::parseOptions(
	                        parameters, TEST_COMMAND));
	CHECK(parameters.parameters.size() == 0);

	CHECK(flagValueCorrect("flaga", true, parsed));
	CHECK(optionValueCorrect("optiona", "foobar", parsed));
	CHECK(flagValueCorrect("flagb", false, parsed));
	CHECK(optionValueCorrect("optionb", "bdefault", parsed));
	CHECK(flagValueCorrect("flagc", true, parsed));
	CHECK(optionValueCorrect("optionc", "barbaz", parsed));
}

TEST_CASE("Test mixed value specification option parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"-A=foobar", "--optionb", "barbaz", "--optionc=foobaz"});

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_NOTHROW(parsed = bdrck::params::detail::parseOptions(
	                        parameters, TEST_COMMAND));
	CHECK(parameters.parameters.size() == 0);

	CHECK(flagValueCorrect("flaga", false, parsed));
	CHECK(optionValueCorrect("optiona", "foobar", parsed));
	CHECK(flagValueCorrect("flagb", false, parsed));
	CHECK(optionValueCorrect("optionb", "barbaz", parsed));
	CHECK(flagValueCorrect("flagc", false, parsed));
	CHECK(optionValueCorrect("optionc", "foobaz", parsed));
}

TEST_CASE("Test arguments left alone during option parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"--flaga", "--optiona", "foobar", "--optionc", "barbaz",
	         "someargument", "-b", "--flagc", "--optionb", "foobaz"});

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_NOTHROW(parsed = bdrck::params::detail::parseOptions(
	                        parameters, TEST_COMMAND));
	CHECK(parameters.parameters.size() == 5);

	CHECK(flagValueCorrect("flaga", true, parsed));
	CHECK(optionValueCorrect("optiona", "foobar", parsed));
	CHECK(flagValueCorrect("flagb", false, parsed));
	CHECK(optionValueCorrect("optionb", "bdefault", parsed));
	CHECK(flagValueCorrect("flagc", false, parsed));
	CHECK(optionValueCorrect("optionc", "barbaz", parsed));
}

TEST_CASE("Test optional option parsing", "[Parameters]")
{
	bdrck::params::ProgramParameters parameters(
	        {"--foo=barbaz", "--rab", "--opta=foobaz"});

	const bdrck::params::Command command(
	        "test", "A command for testing.",
	        bdrck::params::CommandFunction(),
	        {bdrck::params::Option::required("foo", "foo", 'f'),
	         bdrck::params::Option::required("bar", "bar", 'b', "foobar"),
	         bdrck::params::Option::flag("oof", "oof", 'o'),
	         bdrck::params::Option::flag("rab", "rab", 'r'),
	         bdrck::params::Option::optional("opta", "opta"),
	         bdrck::params::Option::optional("optb", "optb")},
	        {}, false);

	std::tuple<bdrck::params::OptionsMap, bdrck::params::FlagsMap> parsed;
	REQUIRE_NOTHROW(parsed = bdrck::params::detail::parseOptions(parameters,
	                                                             command));
	CHECK(parameters.parameters.size() == 0);

	CHECK(optionValueCorrect("foo", "barbaz", parsed));
	CHECK(optionValueCorrect("bar", "foobar", parsed));
	CHECK(flagValueCorrect("oof", false, parsed));
	CHECK(flagValueCorrect("rab", true, parsed));
	CHECK(optionValueCorrect("opta", "foobaz", parsed));
	CHECK(std::get<0>(parsed).find("optb") == std::get<0>(parsed).end());
}
