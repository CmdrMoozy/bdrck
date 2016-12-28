#include <catch/catch.hpp>

#include "bdrck/util/ScopeExit.hpp"

TEST_CASE("Test scope exit utility", "[ScopeExit]")
{
	bool executed = false;

	{
		bdrck::util::ScopeExit se([&executed]() { executed = true; });
	}

	REQUIRE(executed == true);
}
