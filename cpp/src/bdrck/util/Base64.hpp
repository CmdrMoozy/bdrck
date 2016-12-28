#ifndef bdrck_util_Base64_HPP
#define bdrck_util_Base64_HPP

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

namespace bdrck
{
namespace util
{
std::string encodeBase64(void const *data, std::size_t length) noexcept;
std::vector<uint8_t> decodeBase64(std::string const &s);
}
}

#endif
