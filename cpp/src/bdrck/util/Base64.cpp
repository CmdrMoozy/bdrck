#include "Base64.hpp"

#include <algorithm>
#include <cassert>
#include <locale>
#include <map>
#include <sstream>
#include <stdexcept>

namespace
{
inline uint8_t operator"" _b(unsigned long long value)
{
	return static_cast<uint8_t>(value);
}

const std::map<char, uint8_t> DECODE_MAP{
        {'A', 0x00_b}, {'B', 0x01_b}, {'C', 0x02_b}, {'D', 0x03_b},
        {'E', 0x04_b}, {'F', 0x05_b}, {'G', 0x06_b}, {'H', 0x07_b},
        {'I', 0x08_b}, {'J', 0x09_b}, {'K', 0x0A_b}, {'L', 0x0B_b},
        {'M', 0x0C_b}, {'N', 0x0D_b}, {'O', 0x0E_b}, {'P', 0x0F_b},
        {'Q', 0x10_b}, {'R', 0x11_b}, {'S', 0x12_b}, {'T', 0x13_b},
        {'U', 0x14_b}, {'V', 0x15_b}, {'W', 0x16_b}, {'X', 0x17_b},
        {'Y', 0x18_b}, {'Z', 0x19_b}, {'a', 0x1A_b}, {'b', 0x1B_b},
        {'c', 0x1C_b}, {'d', 0x1D_b}, {'e', 0x1E_b}, {'f', 0x1F_b},
        {'g', 0x20_b}, {'h', 0x21_b}, {'i', 0x22_b}, {'j', 0x23_b},
        {'k', 0x24_b}, {'l', 0x25_b}, {'m', 0x26_b}, {'n', 0x27_b},
        {'o', 0x28_b}, {'p', 0x29_b}, {'q', 0x2A_b}, {'r', 0x2B_b},
        {'s', 0x2C_b}, {'t', 0x2D_b}, {'u', 0x2E_b}, {'v', 0x2F_b},
        {'w', 0x30_b}, {'x', 0x31_b}, {'y', 0x32_b}, {'z', 0x33_b},
        {'0', 0x34_b}, {'1', 0x35_b}, {'2', 0x36_b}, {'3', 0x37_b},
        {'4', 0x38_b}, {'5', 0x39_b}, {'6', 0x3A_b}, {'7', 0x3B_b},
        {'8', 0x3C_b}, {'9', 0x3D_b}, {'+', 0x3E_b}, {'/', 0x3F_b},
        {'=', 0x00_b}};

const std::map<uint8_t, char> ENCODE_MAP{
        {0x00_b, 'A'}, {0x01_b, 'B'}, {0x02_b, 'C'}, {0x03_b, 'D'},
        {0x04_b, 'E'}, {0x05_b, 'F'}, {0x06_b, 'G'}, {0x07_b, 'H'},
        {0x08_b, 'I'}, {0x09_b, 'J'}, {0x0A_b, 'K'}, {0x0B_b, 'L'},
        {0x0C_b, 'M'}, {0x0D_b, 'N'}, {0x0E_b, 'O'}, {0x0F_b, 'P'},
        {0x10_b, 'Q'}, {0x11_b, 'R'}, {0x12_b, 'S'}, {0x13_b, 'T'},
        {0x14_b, 'U'}, {0x15_b, 'V'}, {0x16_b, 'W'}, {0x17_b, 'X'},
        {0x18_b, 'Y'}, {0x19_b, 'Z'}, {0x1A_b, 'a'}, {0x1B_b, 'b'},
        {0x1C_b, 'c'}, {0x1D_b, 'd'}, {0x1E_b, 'e'}, {0x1F_b, 'f'},
        {0x20_b, 'g'}, {0x21_b, 'h'}, {0x22_b, 'i'}, {0x23_b, 'j'},
        {0x24_b, 'k'}, {0x25_b, 'l'}, {0x26_b, 'm'}, {0x27_b, 'n'},
        {0x28_b, 'o'}, {0x29_b, 'p'}, {0x2A_b, 'q'}, {0x2B_b, 'r'},
        {0x2C_b, 's'}, {0x2D_b, 't'}, {0x2E_b, 'u'}, {0x2F_b, 'v'},
        {0x30_b, 'w'}, {0x31_b, 'x'}, {0x32_b, 'y'}, {0x33_b, 'z'},
        {0x34_b, '0'}, {0x35_b, '1'}, {0x36_b, '2'}, {0x37_b, '3'},
        {0x38_b, '4'}, {0x39_b, '5'}, {0x3A_b, '6'}, {0x3B_b, '7'},
        {0x3C_b, '8'}, {0x3D_b, '9'}, {0x3E_b, '+'}, {0x3F_b, '/'}};

intmax_t getDecodedSize(std::string const &s)
{
	intmax_t length = static_cast<intmax_t>(s.length());

	if(length < 0)
		return -1;
	if(length == 0)
		return 0;
	if((length % 4) != 0)
		return -1;

	length = (length / 4) * 3;

	intmax_t padding = 0;
	for(auto it = s.rbegin(); it != s.rend(); ++it)
	{
		if(*it == '=')
			++padding;
		else
			break;
	}

	return length - padding;
}
}

namespace bdrck
{
namespace util
{
std::string encodeBase64(void const *data, std::size_t length) noexcept
{
	uint8_t const *bytes = static_cast<uint8_t const *>(data);
	std::ostringstream oss;

	// Add each complete byte tripple to our result.
	for(std::size_t i = 0; (i + 2) < length; i += 3)
	{
		uint8_t values[4] = {
		        static_cast<uint8_t>(bytes[i] >> 2),
		        static_cast<uint8_t>(((bytes[i] & 0x03_b) << 4) |
		                             (bytes[i + 1] >> 4)),
		        static_cast<uint8_t>(((bytes[i + 1] & 0x0F_b) << 2) |
		                             (bytes[i + 2] >> 6)),
		        static_cast<uint8_t>(bytes[i + 2] & 0x3F_b)};

		for(auto const &value : values)
		{
			auto it = ENCODE_MAP.find(value);
			assert(it != ENCODE_MAP.end());
			oss << it->second;
		}
	}

	// Deal with an extra 1 or 2 bytes at the end of the data.
	if((length % 3) == 1)
	{
		uint8_t values[] = {
		        static_cast<uint8_t>(bytes[length - 1] >> 2),
		        static_cast<uint8_t>((bytes[length - 1] & 0x03_b)
		                             << 4)};

		for(auto const &value : values)
		{
			auto it = ENCODE_MAP.find(value);
			assert(it != ENCODE_MAP.end());
			oss << it->second;
		}

		oss << "==";
	}
	else if((length % 3) == 2)
	{
		uint8_t values[] = {
		        static_cast<uint8_t>(bytes[length - 2] >> 2),
		        static_cast<uint8_t>(
		                ((bytes[length - 2] & 0x03_b) << 4) |
		                (bytes[length - 1] >> 4)),
		        static_cast<uint8_t>((bytes[length - 1] & 0x0F_b)
		                             << 2)};

		for(auto const &value : values)
		{
			auto it = ENCODE_MAP.find(value);
			assert(it != ENCODE_MAP.end());
			oss << it->second;
		}

		oss << '=';
	}

	return oss.str();
}

std::vector<uint8_t> decodeBase64(std::string const &s)
{
	std::locale locale;
	std::string stripped(s);
	stripped.erase(std::remove_if(stripped.begin(), stripped.end(),
	                              [&locale](char const &c) -> bool {
		                              return std::isspace(c, locale);
		                      }),
	               stripped.end());

	intmax_t length = getDecodedSize(stripped);
	if(length == -1)
	{
		throw std::runtime_error(
		        "Cannot decode an invalid base-64 string.");
	}
	std::vector<uint8_t> ret(
	        static_cast<std::vector<uint8_t>::size_type>(length), 0);

	for(std::size_t i = 0; (i + 3) < stripped.length(); i += 4)
	{
		// Get bit values for the input base-64 characters.
		auto ait = DECODE_MAP.find(stripped[i]);
		auto bit = DECODE_MAP.find(stripped[i + 1]);
		auto cit = DECODE_MAP.find(stripped[i + 2]);
		auto dit = DECODE_MAP.find(stripped[i + 3]);
		if((ait == DECODE_MAP.end()) || (bit == DECODE_MAP.end()) ||
		   (cit == DECODE_MAP.end()) || (dit == DECODE_MAP.end()))
		{
			throw std::runtime_error(
			        "Cannot decode an invalid base-64 string.");
		}
		uint8_t a = ait->second;
		uint8_t b = bit->second;
		uint8_t c = cit->second;
		uint8_t d = dit->second;

		// Compute the final full byte values.
		uint8_t av = static_cast<uint8_t>((a << 2) | (b >> 4));
		uint8_t bv =
		        static_cast<uint8_t>(((b & 0x0F_b) << 4) | (c >> 2));
		uint8_t cv = static_cast<uint8_t>(((c & 0x03_b) << 6) | d);

		// Place these final values in the return vector.
		std::size_t retOffset = (i / 4) * 3;
		ret[retOffset] = av;
		if(stripped[i + 2] != '=')
			ret[retOffset + 1] = bv;
		if(stripped[i + 3] != '=')
			ret[retOffset + 2] = cv;
	}

	return ret;
}
}
}
