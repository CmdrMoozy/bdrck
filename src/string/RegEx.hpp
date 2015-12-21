#ifndef bdrck_string_RegEx_HPP
#define bdrck_string_RegEx_HPP

#include <memory>
#include <string>
#include <vector>

#include "bdrck/string/StringRef.hpp"

namespace bdrck
{
namespace string
{
namespace detail
{
struct RegExImpl;
}

struct RegExOptions
{
};

struct RegExResult
{
	bool matched;
	std::vector<StringRef> matches;
};

class RegEx
{
public:
	RegEx(std::string const &pattern, RegExOptions const &options = {});

	RegEx(RegEx const &o);
	RegEx(RegEx &&) = default;
	RegEx &operator=(RegEx const &o);
	RegEx &operator=(RegEx &&) = default;

	~RegEx() = default;

	RegExResult match(StringRef const &text) const;

private:
	std::unique_ptr<detail::RegExImpl> impl;
};
}
}

#endif
