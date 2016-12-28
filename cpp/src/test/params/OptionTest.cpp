#include <catch/catch.hpp>

#include <boost/optional/optional.hpp>

#include "bdrck/params/Option.hpp"

TEST_CASE("Test option default value construction", "[Parameters]")
{
	boost::optional<bdrck::params::Option> option;
	CHECK_NOTHROW(option = bdrck::params::Option::required(
	                      "foobar", "A test option.", 'f', "barbaz"));
}

TEST_CASE("Test default constructed option set iterator equality",
          "[Parameters]")
{
	bdrck::params::OptionSetConstIterator a;
	bdrck::params::OptionSetConstIterator b;
	CHECK(a == b);
	++a;
	CHECK(a == b);
	++b;
	CHECK(a == b);
}

TEST_CASE("Test option set iterating", "[Parameters]")
{
	const std::initializer_list<bdrck::params::Option> optionsList{
	        bdrck::params::Option::required("foo", ""),
	        bdrck::params::Option::required("bar", ""),
	        bdrck::params::Option::required("baz", ""),
	        bdrck::params::Option::required("zab", ""),
	        bdrck::params::Option::required("rab", ""),
	        bdrck::params::Option::required("oof", ""),
	        bdrck::params::Option::required("foobar", ""),
	        bdrck::params::Option::required("barbaz", ""),
	        bdrck::params::Option::required("zabrab", ""),
	        bdrck::params::Option::required("raboof", "")};
	bdrck::params::OptionSet options(optionsList);
	CHECK(optionsList.size() == options.size());
	CHECK(optionsList.size() ==
	      std::distance(options.begin(), options.end()));

	auto expIt = optionsList.begin();
	for(auto it = options.begin(); it != options.end(); ++it)
	{
		REQUIRE(expIt != optionsList.end());
		CHECK((*expIt).name == (*it).name);
		++expIt;
	}
}

namespace
{
bool findSuccessful(bdrck::params::OptionSet const &options,
                    std::string const &parameter,
                    std::string const &expectedName)
{
	bdrck::params::Option const *option = options.find(parameter);
	if(option == nullptr)
		return false;
	return option->name == expectedName;
}
}

TEST_CASE("Test option set finding", "[Parameters]")
{
	bdrck::params::OptionSet options{
	        bdrck::params::Option::required("foo", "", 'o'),
	        bdrck::params::Option::required("bar", "", 'r'),
	        bdrck::params::Option::flag("baz", "", 'z'),
	        bdrck::params::Option::flag("zab", "", 'Z'),
	        bdrck::params::Option::required("rab", "", 'R'),
	        bdrck::params::Option::required("oof", "", 'O'),
	        bdrck::params::Option::required("foobar", "", 'f'),
	        bdrck::params::Option::flag("barbaz", "", 'b'),
	        bdrck::params::Option::flag("zabrab", "", 'B'),
	        bdrck::params::Option::required("raboof", "", 'F')};

	CHECK(findSuccessful(options, "foo", "foo"));
	CHECK(findSuccessful(options, "o", "foo"));
	CHECK(findSuccessful(options, "bar", "bar"));
	CHECK(findSuccessful(options, "r", "bar"));
	CHECK(findSuccessful(options, "baz", "baz"));
	CHECK(findSuccessful(options, "z", "baz"));
	CHECK(findSuccessful(options, "zab", "zab"));
	CHECK(findSuccessful(options, "Z", "zab"));
	CHECK(findSuccessful(options, "rab", "rab"));
	CHECK(findSuccessful(options, "R", "rab"));
	CHECK(findSuccessful(options, "oof", "oof"));
	CHECK(findSuccessful(options, "O", "oof"));
	CHECK(findSuccessful(options, "foobar", "foobar"));
	CHECK(findSuccessful(options, "f", "foobar"));
	CHECK(findSuccessful(options, "barbaz", "barbaz"));
	CHECK(findSuccessful(options, "b", "barbaz"));
	CHECK(findSuccessful(options, "zabrab", "zabrab"));
	CHECK(findSuccessful(options, "B", "zabrab"));
	CHECK(findSuccessful(options, "raboof", "raboof"));
	CHECK(findSuccessful(options, "F", "raboof"));

	CHECK(!findSuccessful(options, "foo", "bar"));
	CHECK(!findSuccessful(options, "syn", "syn"));
	CHECK(!findSuccessful(options, "s", "syn"));
	CHECK(!findSuccessful(options, "ack", "ack"));
	CHECK(!findSuccessful(options, "a", "ack"));
	CHECK(!findSuccessful(options, "-", "foobar"));
}
