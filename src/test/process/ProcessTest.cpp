#include <catch/catch.hpp>

#include <cstring>
#include <string>

#include "bdrck/fs/Util.hpp"
#include "bdrck/process/Pipe.hpp"
#include "bdrck/process/Process.hpp"

TEST_CASE("Verify that process launching works", "[Process]")
{
	// By definition, on Linux the exit code must be in the range
	// [0, 255]. Other values are returned modulo 256.
	constexpr int TEST_EXIT_CODE = 137;
	constexpr char const *TEST_STRING = "this is a test";

	const std::string TEST_ECHO_BINARY = bdrck::fs::combinePaths(
	        bdrck::fs::getCurrentDirectory(), "bdrck-test-echo");
	REQUIRE(bdrck::fs::isExecutable(TEST_ECHO_BINARY));

	bdrck::process::Process child(
	        TEST_ECHO_BINARY,
	        {"-1", "-2", "-e", std::to_string(TEST_EXIT_CODE)});

	std::size_t written = bdrck::process::pipe::write(
	        child.getPipe(bdrck::process::StdStream::STDIN), TEST_STRING,
	        std::strlen(TEST_STRING));
	REQUIRE(written == std::strlen(TEST_STRING));
	child.closePipe(bdrck::process::StdStream::STDIN);

	std::string out = bdrck::process::pipe::readAll(
	        child.getPipe(bdrck::process::StdStream::STDOUT));
	std::string err = bdrck::process::pipe::readAll(
	        child.getPipe(bdrck::process::StdStream::STDERR));
	int ret = child.wait();

	CHECK(out == TEST_STRING);
	CHECK(err == TEST_STRING);
	CHECK(ret == TEST_EXIT_CODE);
}
