#include "Iterator.hpp"

#include <stdexcept>

#include <boost/filesystem.hpp>

#include "bdrck/fs/Util.hpp"
#include "bdrck/util/Error.hpp"

namespace bdrck
{
namespace fs
{
namespace detail
{
struct IteratorImpl
{
	boost::system::error_code error;
	boost::filesystem::recursive_directory_iterator iterator;

	IteratorImpl(std::string const &p, bool fs)
	        : error(),
	          iterator(p,
	                   fs ? boost::filesystem::symlink_option::recurse
	                      : boost::filesystem::symlink_option::no_recurse,
	                   error)
	{
		if(error)
			throw std::runtime_error(error.message());
	}

	~IteratorImpl()
	{
	}

	bool isEnd() const
	{
		return iterator ==
		       boost::filesystem::recursive_directory_iterator();
	}
};
}
}
}

namespace
{
boost::optional<std::string>
currentOf(bdrck::fs::detail::IteratorImpl const &impl)
{
	boost::filesystem::recursive_directory_iterator endIt;
	if(impl.iterator == endIt)
		return boost::none;
	return bdrck::fs::normalizePath(impl.iterator->path().string());
}
}

namespace bdrck
{
namespace fs
{
Iterator::Iterator() : impl(nullptr), current(boost::none)
{
}

Iterator::Iterator(std::string const &p, bool followSymlinks)
        : impl(std::make_shared<detail::IteratorImpl>(p, followSymlinks)),
          first(bdrck::fs::normalizePath(p)),
          current(currentOf(*impl))
{
}

bool Iterator::operator==(Iterator const &o) const
{
	if(!!impl && !!o.impl)
	{
		if(impl != o.impl)
			return false;
	}

	return first == o.first && current == o.current;
}

bool Iterator::operator!=(Iterator const &o) const
{
	return !(*this == o);
}

std::string const &Iterator::operator*() const
{
	if(!!first)
		return *first;
	else
		return *current;
}

std::string const *Iterator::operator->() const
{
	if(!!first)
		return &(*first);
	else
		return &(*current);
}

Iterator &Iterator::operator++()
{
	// If we are at the first entry, unset it without iterating. This
	// is a semi-hacky way to get recursive_directory_iterator to visit
	// the initial directory before recursing into it.
	if(!!first)
	{
		first = boost::none;
		return *this;
	}

	// Don't try to recurse into broken symlinks.
	if(impl->iterator->symlink_status().type() ==
	   boost::filesystem::symlink_file)
	{
		if(impl->iterator->status().type() ==
		   boost::filesystem::file_not_found)
		{
			impl->iterator.no_push(true);
		}
	}

	++(impl->iterator);
	current = currentOf(*impl);
	return *this;
}
}
}
