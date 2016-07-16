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
	typedef Wrapper<git_index, git_index_free> base_type;

public:
	Index(Repository &repository);

	Index(Index const &) = delete;
	Index(Index &&) = default;
	Index &operator=(Index const &) = delete;
	Index &operator=(Index &&) = default;

	std::size_t getEntryCount() const;

	void addAll(StrArray const &pathspec);

	/**
	 * Write the current index to disk as a tree, and return the OID of
	 * this tree. This is the OID that can be used, for example, to create
	 * a commit.
	 *
	 * \return The OID of the newly writtent ree.
	 */
	git_oid writeTree();
};
}
}

#endif
