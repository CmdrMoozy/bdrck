#ifndef bdrck_git_Index_HPP
#define bdrck_git_Index_HPP

#include <cstddef>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;
class StrArray;

class Index : public Wrapper<git_index, git_index_free>
{
private:
	typedef Wrapper<git_repository, git_repository_free> base_type;

public:
	Index(Repository &repository);

	std::size_t getEntryCount() const;

	void addAll(StrArray const &pathspec);
};
}
}

#endif
