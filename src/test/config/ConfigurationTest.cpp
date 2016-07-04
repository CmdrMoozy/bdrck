#include <catch/catch.hpp>

#include <mutex>

#include "bdrck/config/Configuration.hpp"
#include "bdrck/fs/TemporaryStorage.hpp"

#include "TestConfiguration.pb.h"

namespace
{
bdrck::test::messages::TestConfiguration getDefaults()
{
	static std::mutex mutex;
	static bool initialized{false};
	static bdrck::test::messages::TestConfiguration defaults;

	std::lock_guard<std::mutex> lock(mutex);
	if(!initialized)
	{
		initialized = true;
		defaults.set_foo("ABC");
		defaults.set_bar("DEF");
	}

	return defaults;
}

struct TestContext
{
	bdrck::fs::TemporaryStorage file;
	const bdrck::config::ConfigurationIdentifier identifier;
	bdrck::config::ConfigurationInstance<
	        bdrck::test::messages::TestConfiguration>
	        instanceHandle;
	bdrck::config::Configuration<bdrck::test::messages::TestConfiguration>
	        &instance;

	TestContext()
	        : file(bdrck::fs::TemporaryStorageType::FILE),
	          identifier({"bdrck", "ConfigurationTest"}),
	          instanceHandle(identifier, getDefaults(), file.getPath()),
	          instance(bdrck::config::Configuration<
	                   bdrck::test::messages::TestConfiguration>::
	                           instance(identifier))
	{
	}
};
}

TEST_CASE("Test setting values", "[Configuration]")
{
	TestContext context;
	CHECK(context.instance.get().foo() == "ABC");
	auto message = context.instance.get();
	message.set_foo("quux");
	context.instance.set(message);
	CHECK(context.instance.get().foo() == "quux");
}

TEST_CASE("Test default values", "[Configuration]")
{
	TestContext context;

	CHECK(context.instance.get().foo() == "ABC");
	CHECK(context.instance.get().bar() == "DEF");

	auto message = context.instance.get();
	message.set_baz("oof");
	context.instance.set(message);
	CHECK(context.instance.get().baz() == "oof");

	message = context.instance.get();
	message.set_foo("XYZ");
	message.set_bar("ZYX");
	message.set_quux("xuuq");
	context.instance.set(message);
	REQUIRE(context.instance.get().foo() == "XYZ");
	REQUIRE(context.instance.get().bar() == "ZYX");
	context.instance.resetAll();
	CHECK(context.instance.get().foo() == "ABC");
	CHECK(context.instance.get().bar() == "DEF");
}

TEST_CASE("Test configuration modification signal", "[Configuration]")
{
	TestContext context;

	std::vector<std::string> changedCalls;
	auto connection = context.instance.handleConfigurationFieldChanged(
	        [&changedCalls](std::string const &field) {
		        changedCalls.push_back(field);
		});

	auto message = context.instance.get();
	message.set_foo("aaa");
	message.set_bar("bbb");
	message.mutable_sub()->set_foobar("ccc");
	message.mutable_sub()->set_barbaz("ddd");
	context.instance.set(message);
	CHECK(changedCalls == std::vector<std::string>({"foo", "bar", "sub"}));
	changedCalls.clear();

	context.instance.resetAll();
	CHECK(changedCalls == std::vector<std::string>({"foo", "bar", "sub"}));
	changedCalls.clear();
}
