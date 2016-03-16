#ifndef bdrck_json_generate_HPP
#define bdrck_json_generate_HPP

#include <ostream>

#include <boost/optional/optional.hpp>

#include "bdrck/json/Types.hpp"

namespace bdrck
{
namespace json
{
void generate(std::ostream &out, boost::optional<JsonValue> const &contents,
              bool beautify = false);
}
}

#endif
