#include "Iterator.hpp"

#include <cassert>
#include <cerrno>
#include <cstring>
#include <functional>
#include <stdexcept>

#include <fts.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>

#include <bdrck/fs/Util.hpp>
#include <bdrck/util/Error.hpp>

namespace
{
int compareFtsEntries(const FTSENT **a, const FTSENT **b)
{
	int ret = std::strcmp((*a)->fts_path, (*b)->fts_path);
	if(ret == 0)
		ret = std::strcmp((*a)->fts_name, (*b)->fts_name);
	return ret;
}
}

namespace bdrck
{
namespace fs
{
namespace detail
{
constexpr int DEFAULT_FTS_OPTIONS = FTS_COMFOLLOW | FTS_NOCHDIR;

struct IteratorImpl
{
	const bool followSymlinks;
	const bool sort;

	std::unique_ptr<char, std::function<void(char *)>> path;
	std::vector<char *> paths;
	FTS *state;

	IteratorImpl(std::string const &p, bool fs, bool s)
	        : followSymlinks(fs),
	          sort(s),
	          path(strndup(p.data(), p.length()),
	               [](char *ptr)
	               {
		               std::free(ptr);
		       }),
	          paths({path.get(), nullptr}),
	          state(fts_open(
	                  paths.data(),
	                  DEFAULT_FTS_OPTIONS |
	                          (followSymlinks ? FTS_LOGICAL : FTS_PHYSICAL),
	                  sort ? &compareFtsEntries : nullptr))
	{
		assert(state != nullptr);
	}

	~IteratorImpl()
	{
		int ret = fts_close(state);
		assert(ret == 0);
	}
};
}
}
}

namespace
{
std::experimental::optional<std::string>
next(bdrck::fs::detail::IteratorImpl &impl)
{
	while(true)
	{
		FTSENT *entry = fts_read(impl.state);
		int error = errno;
		if(entry == nullptr)
		{
			if(error != 0)
				bdrck::util::error::throwErrnoError(error);

			return std::experimental::nullopt;
		}

		switch(entry->fts_info)
		{
		case FTS_DNR:
		case FTS_ERR:
		case FTS_NS:
			bdrck::util::error::throwErrnoError(entry->fts_errno);

		case FTS_DC:
			throw std::runtime_error("Encountered cyclical link "
			                         "while iterating over "
			                         "directory.");

		case FTS_DP:
			// We do not want to visit parent directories twice.
			// Skip the postorder traversal.
			continue;
		}

		return std::string(entry->fts_path);
	}
}
}

namespace bdrck
{
namespace fs
{
Iterator::Iterator() : impl(nullptr), current(std::experimental::nullopt)
{
}

Iterator::Iterator(std::string const &p, bool followSymlinks, bool sort)
        : impl(std::make_shared<detail::IteratorImpl>(p, followSymlinks, sort)),
          current(next(*impl))
{
}

bool Iterator::operator==(Iterator const &o) const
{
	if(!!impl && !!o.impl)
	{
		if(impl != o.impl)
			return false;
	}

	return current == o.current;
}

bool Iterator::operator!=(Iterator const &o) const
{
	return !(*this == o);
}

std::string const &Iterator::operator*() const
{
	return *current;
}

std::string const *Iterator::operator->() const
{
	return &(*current);
}

Iterator &Iterator::operator++()
{
	current = next(*impl);
	return *this;
}
}
}
