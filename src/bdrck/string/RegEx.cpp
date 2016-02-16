#include "RegEx.hpp"

#include <algorithm>
#include <cassert>
#include <cstddef>
#include <iterator>
#include <stdexcept>

#include <boost/optional/optional.hpp>

#include <re2/re2.h>
#include <re2/stringpiece.h>

namespace bdrck
{
namespace string
{
namespace detail
{
struct RegExImpl
{
	RegExOptions options;
	boost::optional<re2::RE2> regex;

	RegExImpl(std::string const &p, RegExOptions const &);

	RegExImpl(RegExImpl const &o);
	RegExImpl(RegExImpl &&) = default;
	RegExImpl &operator=(RegExImpl const &o);
	RegExImpl &operator=(RegExImpl &&) = default;
};

RegExImpl::RegExImpl(std::string const &p, RegExOptions const &) : regex()
{
	regex.emplace(p);
	if(!regex->ok())
		throw std::runtime_error(regex->error());
}

RegExImpl::RegExImpl(RegExImpl const &o)
{
	*this = o;
}

RegExImpl &RegExImpl::operator=(RegExImpl const &o)
{
	if(this == &o)
		return *this;
	regex.emplace((*o.regex).pattern());
	return *this;
}
}

RegEx::RegEx(std::string const &pattern, RegExOptions const &options)
        : impl(new detail::RegExImpl(pattern, options))
{
}

RegEx::RegEx(RegEx const &o)
{
	*this = o;
}

RegEx &RegEx::operator=(RegEx const &o)
{
	if(this == &o)
		return *this;
	impl.reset();
	if(!!o.impl)
		impl.reset(new detail::RegExImpl(*o.impl));
	return *this;
}

RegEx::~RegEx()
{
}

RegExResult RegEx::match(StringRef const &text) const
{
	assert(!!impl);
	assert(!!impl->regex);

	std::vector<re2::StringPiece> matches(static_cast<std::size_t>(
	        impl->regex->NumberOfCapturingGroups() + 1));
	bool matched = impl->regex->Match(
	        re2::StringPiece(text.data(), static_cast<int>(text.size())), 0,
	        static_cast<int>(text.size()), re2::RE2::Anchor::UNANCHORED,
	        matches.data(), static_cast<int>(matches.size()));

	RegExResult result = {matched, {}};
	if(!matched)
		return result;
	result.matches.reserve(matches.size());
	std::transform(matches.begin(), matches.end(),
	               std::back_inserter(result.matches),
	               [](re2::StringPiece const &piece)
	               {
		return StringRef(piece.data(),
		                 static_cast<std::size_t>(piece.size()));
	});
	return result;
}
}
}
