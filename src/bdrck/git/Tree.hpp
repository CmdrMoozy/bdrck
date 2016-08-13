#ifndef bdrck_git_Tree_HPP
#define bdrck_git_Tree_HPP

#include <functional>
#include <string>

#include <git2.h>

#include "bdrck/git/Oid.hpp"
#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Commit;
class Object;
class Repository;

class Tree : public Wrapper<git_tree, git_tree_free>
{
private:
	typedef Wrapper<git_tree, git_tree_free> base_type;

public:
	explicit Tree(Object const &obj);
	explicit Tree(Commit const &commit);
	Tree(Repository &repository, Oid const &id);

	Tree(Tree const &) = delete;
	Tree(Tree &&) = default;
	Tree &operator=(Tree const &) = delete;
	Tree &operator=(Tree &&) = default;

	~Tree() = default;

	Oid getId() const;

	/**
	 * Iterate over all entries in this tree, calling the given callback on
	 * each one.
	 *
	 * The callback should return false if the traversal should stop.
	 * Otherwise, it should return true and the traversal will continue.
	 * The path passed to the callback will be the path of the current tree
	 * entry, relative to the repository's working directory.
	 *
	 * The filemode filter is a series of git_filemode_t entries OR'ed
	 * together. The callback will only be called on tree entries whose
	 * filemode matches one of the flags set in this filter. By default,
	 * all tree entries are included.
	 */
	void walk(std::function<bool(std::string const &)> const &callback,
	          int filemodeFilter = ~git_filemode_t(0)) const;
};
}
}

#endif
