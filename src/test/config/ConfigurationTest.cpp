#include <catch/catch.hpp>

#include <cstdint>
#include <initializer_list>
#include <limits>
#include <utility>

#include "bdrck/config/Configuration.hpp"
#include "bdrck/config/deserialize.hpp"
#include "bdrck/config/serialize.hpp"
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
