#ifndef bdrck_json_Types_HPP
#define bdrck_json_Types_HPP

#include <cstdint>
#include <map>
#include <vector>

#include <boost/variant/get.hpp>
#include <boost/variant/recursive_variant.hpp>
#include <boost/variant/recursive_wrapper.hpp>
#include <boost/variant/variant.hpp>

#include "bdrck/string/StringRef.hpp"

namespace bdrck
{
namespace json
{
struct NullType
{
	NullType();

	NullType(NullType const &) = default;
	NullType(NullType &&) = default;
	NullType &operator=(NullType const &) = default;
	NullType &operator=(NullType &&) = default;

	~NullType() = default;

	bool operator==(NullType const &o) const;
	bool operator!=(NullType const &o) const;
	bool operator<(NullType const &o) const;
	bool operator<=(NullType const &o) const;
	bool operator>(NullType const &o) const;
	bool operator>=(NullType const &o) const;
};

typedef bool BooleanType;
typedef long long int IntegerType;
typedef double DoubleType;
typedef std::vector<uint8_t> StringType;

typedef boost::make_recursive_variant<
        NullType, BooleanType, IntegerType, DoubleType, StringType,
        std::map<StringType, boost::recursive_variant_>,
        std::vector<boost::recursive_variant_>>::type JsonValue;

typedef std::map<StringType, JsonValue> MapType;
typedef std::vector<JsonValue> ArrayType;

template <typename JsonType> JsonType &get(JsonValue &value)
{
	return boost::get<JsonType>(value);
}

template <typename JsonType> JsonType const &get(JsonValue const &value)
{
	return boost::get<JsonType>(value);
}

bdrck::string::StringRef toString(StringType const &s);
StringType fromString(bdrck::string::StringRef const &s);
}
}

#endif
