#ifndef bdrck_git_StrArray_HPP
#define bdrck_git_StrArray_HPP

#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <iterator>
#include <memory>
#include <stdexcept>
#include <vector>

#include <git2.h>

#include "bdrck/util/Error.hpp"

namespace bdrck
{
namespace git
{
class StrArray
{
private:
	typedef std::unique_ptr<char, std::function<void(char *)>> CStringPtr;

public:
	template <typename Iterator> StrArray(Iterator begin, Iterator end);

	StrArray(StrArray const &) = default;
	StrArray(StrArray &&) = default;
	StrArray &operator=(StrArray const &) = default;
	StrArray &operator=(StrArray &&) = default;

	~StrArray() = default;

	git_strarray &get();
	git_strarray const &get() const;

private:
	std::vector<CStringPtr> strings;
	std::vector<char *> unwrappedStrings;
	git_strarray strarray;
};

template <typename Iterator>
StrArray::StrArray(Iterator begin, Iterator end)
        : strings(), unwrappedStrings(), strarray({nullptr, 0})
{
	for(auto it = begin; it != end; ++it)
	{
		char *str = strdup(it->c_str());
		if(str == nullptr)
			bdrck::util::error::throwErrnoError();
		strings.emplace_back(str, [](char *s)
		                     {
			std::free(s);
		});
	}

	std::transform(strings.begin(), strings.end(),
	               std::back_inserter(unwrappedStrings),
	               [](CStringPtr & s) -> char *
	               {
		return s.get();
	});

	strarray.strings = unwrappedStrings.data(),
	strarray.count = unwrappedStrings.size();
}
}
}

#endif
