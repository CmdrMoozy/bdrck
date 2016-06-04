#include <catch/catch.hpp>

#include <cstdint>
#include <initializer_list>
#include <limits>
#include <set>
#include <string>
#include <utility>

#include "bdrck/config/Configuration.hpp"
#include "bdrck/config/deserialize.hpp"
#include "bdrck/config/serialize.hpp"
#include "bdrck/fs/TemporaryStorage.hpp"
#include "bdrck/util/floatCompare.hpp"

namespace
{
template <typename T> std::pair<T, T> performRoundTripTest(T const &v)
{
	using namespace bdrck::config;
	std::string serialized = serialize(v);
	return std::make_pair(v, bdrck::config::deserialize<T>(serialized));
}
}

TEST_CASE("Test integer round tripping", "[Configuration]")
{
	const std::initializer_list<uint64_t> TEST_CASES{
	        0, std::numeric_limits<uint64_t>::max() / 2,
	        std::numeric_limits<uint64_t>::max()};

	for(auto const &v : TEST_CASES)
	{
		auto pair = performRoundTripTest(v);
		CHECK(pair.first == pair.second);
	}
}

TEST_CASE("Test floating point round tripping", "[Configuration]")
{
	const std::initializer_list<double> TEST_CASES{0.0, -12345.54321,
	                                               12345.54321};

	for(auto const &v : TEST_CASES)
	{
		auto pair = performRoundTripTest(v);
		CHECK(bdrck::util::floatCompare(pair.first, pair.second) == 0);
	}
}

TEST_CASE("Test boolean round tripping", "[Configuration]")
{
	const std::initializer_list<bool> TEST_CASES{true, false};

	for(auto const &v : TEST_CASES)
	{
		auto pair = performRoundTripTest(v);
		CHECK(pair.first == pair.second);
	}
}

TEST_CASE("Test mutation and retrieval functions", "[Configuration]")
{
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE);
	const bdrck::config::ConfigurationIdentifier identifier{
	        "bdrck", "ConfigurationTest"};
	bdrck::config::ConfigurationInstance instanceHandle(identifier, {},
	                                                    file.getPath());
	bdrck::config::Configuration &instance =
	        bdrck::config::Configuration::instance(identifier);

	REQUIRE(instance.empty());

	REQUIRE(!instance.contains("some bad key"));
	CHECK(instance.get("some bad key", std::string("baz")) == "baz");

	REQUIRE(!instance.contains("foo"));
	instance.set("foo", "bar");
	REQUIRE(instance.contains("foo"));
	CHECK(instance.get("foo") == "bar");

	instance.remove("foo");
	CHECK(!instance.contains("foo"));

	instance.set("oof", "rab");
	REQUIRE(!instance.empty());
	instance.clear();
	CHECK(instance.empty());
}

TEST_CASE("Test default value functions", "[Configuration]")
{
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE);
	const bdrck::config::ConfigurationIdentifier identifier{
	        "bdrck", "ConfigurationTest"};

	{
		bdrck::config::ConfigurationInstance instanceHandle(
		        identifier, {}, file.getPath());
		bdrck::config::Configuration &instance =
		        bdrck::config::Configuration::instance(identifier);

		REQUIRE(instance.empty());
		instance.set("bar", "quux");
		REQUIRE(instance.get("bar") == "quux");
	}

	bdrck::config::ConfigurationInstance instanceHandle(
	        identifier, {bdrck::config::makeDefault("foo", "zab"),
	                     bdrck::config::makeDefault("bar", "rab")},
	        file.getPath());
	bdrck::config::Configuration &instance =
	        bdrck::config::Configuration::instance(identifier);

	REQUIRE(instance.contains("foo"));
	CHECK(instance.get("foo") == "zab");
	REQUIRE(instance.contains("bar"));
	CHECK(instance.get("bar") == "quux");

	REQUIRE(!instance.contains("baz"));
	instance.set("baz", "oof");
	REQUIRE(instance.contains("baz"));
	CHECK(instance.get("baz") == "oof");
	instance.reset("baz");
	CHECK(!instance.contains("baz"));

	REQUIRE(!instance.contains("quux"));
	instance.set("quux", "xuuq");
	REQUIRE(instance.contains("quux"));
	instance.resetAll();
	CHECK(instance.get("foo") == "zab");
	CHECK(instance.get("bar") == "rab");
	CHECK(!instance.contains("baz"));
	CHECK(!instance.contains("quux"));
}

TEST_CASE("Test configuration modification signal", "[Configuration]")
{
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE);
	const bdrck::config::ConfigurationIdentifier identifier{
	        "bdrck", "ConfigurationTest"};
	bdrck::config::ConfigurationInstance instanceHandle(
	        identifier, {bdrck::config::makeDefault("foo", "quux"),
	                     bdrck::config::makeDefault("bar", false)},
	        file.getPath());
	bdrck::config::Configuration &instance =
	        bdrck::config::Configuration::instance(identifier);

	std::set<std::string> changedCalls;
	auto connection = instance.handleConfigurationChanged([&changedCalls](
	        std::string const &key) { changedCalls.insert(key); });

	instance.set("foo", "foo");
	instance.setFrom("bar", true);
	instance.set("baz", "baz");
	instance.set("quux", "quux");
	instance.set("oof", "oof");
	CHECK(changedCalls ==
	      std::set<std::string>({"foo", "bar", "baz", "quux", "oof"}));
	changedCalls.clear();

	instance.remove("quux");
	CHECK(changedCalls == std::set<std::string>({"quux"}));
	changedCalls.clear();

	instance.reset("oof");
	CHECK(changedCalls == std::set<std::string>({"oof"}));
	changedCalls.clear();

	instance.resetAll();
	CHECK(changedCalls == std::set<std::string>({"foo", "bar", "baz"}));
	changedCalls.clear();

	instance.clear();
	CHECK(changedCalls == std::set<std::string>({"foo", "bar"}));
	changedCalls.clear();
}

TEST_CASE("Test configuration vector handling", "[Configuration]")
{
	bdrck::fs::TemporaryStorage file(bdrck::fs::TemporaryStorageType::FILE);
	const bdrck::config::ConfigurationIdentifier identifier{
	        "bdrck", "ConfigurationTest"};
	bdrck::config::ConfigurationInstance instanceHandle(identifier, {},
	                                                    file.getPath());
	bdrck::config::Configuration &instance =
	        bdrck::config::Configuration::instance(identifier);

	const std::vector<bool> TEST_CASE{false, true,  false, false,
	                                  true,  false, true};
	instance.setAllFrom("foo", TEST_CASE);
	auto result = instance.getAllAs<bool>("foo");
	CHECK(TEST_CASE == result);
}
