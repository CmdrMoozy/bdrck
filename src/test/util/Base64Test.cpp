#include <catch/catch.hpp>

#include <cstdint>
#include <string>
#include <utility>
#include <vector>

#include "bdrck/util/Base64.hpp"

namespace
{
const std::vector<std::pair<std::string, std::string>> TEST_VECTORS{
        {"", ""},
        {"f", "Zg=="},
        {"fo", "Zm8="},
        {"foo", "Zm9v"},
        {"foob", "Zm9vYg=="},
        {"fooba", "Zm9vYmE="},
        {"foobar", "Zm9vYmFy"}};
}

TEST_CASE("Test base-64 encoding", "[Base64]")
{
	for(auto const &vector : TEST_VECTORS)
	{
		std::string encoded = bdrck::util::encodeBase64(
		        vector.first.data(), vector.first.length());
		CHECK(vector.second == encoded);
	}
}

TEST_CASE("Test base-64 decoding", "[Base64]")
{
	for(auto const &vector : TEST_VECTORS)
	{
		std::vector<uint8_t> decoded =
		        bdrck::util::decodeBase64(vector.second);
		std::string decodedString(
		        reinterpret_cast<char const *>(decoded.data()),
		        decoded.size());
		CHECK(vector.first == decodedString);
	}
}
