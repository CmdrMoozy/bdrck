#ifndef bdrck_json_parse_HPP
#define bdrck_json_parse_HPP

#include <functional>
#include <istream>
#include <string>

#include <boost/optional/optional.hpp>

#include "bdrck/json/Types.hpp"

namespace bdrck
{
namespace json
{
struct ParseCallbacks
{
	std::function<bool()> nullCallback;
	std::function<bool(BooleanType)> booleanCallback;
	std::function<bool(IntegerType)> integerCallback;
	std::function<bool(DoubleType)> doubleCallback;
	std::function<bool(StringType const &)> stringCallback;
	std::function<bool()> startMapCallback;
	std::function<bool(StringType const &)> mapKeyCallback;
	std::function<bool()> endMapCallback;
	std::function<bool()> startArrayCallback;
	std::function<bool()> endArrayCallback;
};

void parse(std::istream &in, ParseCallbacks &callbacks);
boost::optional<JsonValue> parseAll(std::istream &in);
}
}

#endif
