#include "UUID.hpp"

#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <random>
#include <sstream>
#include <vector>

namespace
{
constexpr std::size_t UUID_BYTE_LENGTH = 16;
}

namespace bdrck
{
namespace util
{
std::string generateUUID()
{
	std::random_device rd;
	std::mt19937_64 engine(rd());
	std::uniform_int_distribution<uint8_t> dist;

	// Generate a series of random bytes.
	std::vector<uint8_t> bytes(UUID_BYTE_LENGTH);
	for(auto &byte : bytes)
		byte = dist(engine);

	// To be a valid version 4 UUID, byte 6 must start with 0x4, and byte
	// 8 must start with 0b10. Apply such a mask to these bytes.
	bytes[6] = 0x40 | (bytes[6] & 0x0F);
	bytes[8] = 0x80 | (bytes[8] & 0x3F);

	// Print the UUID hex components. 2 hex characters per byte.
	char hex[UUID_BYTE_LENGTH * 2 + 1];
	for(std::size_t i = 0; i < UUID_BYTE_LENGTH; ++i)
	{
		int ret = snprintf(&hex[i * 2], 3, "%02x", bytes[i]);
		assert(ret == 2);
	}

	// Add '-' characters to form the final UUID. The components are
	// 8, 4, 4, 4, and 12 characters long, respectively.
	std::ostringstream oss;
	oss << std::string(&hex[0], 8) << "-" << std::string(&hex[8], 4) << "-"
	    << std::string(&hex[12], 4) << "-" << std::string(&hex[16], 4)
	    << "-" << std::string(&hex[20], 12);
	return oss.str();
}
}
}
