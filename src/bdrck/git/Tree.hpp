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
	explicit Tree(Object const &object);
	explicit Tree(Commit const &commit);
	Tree(Repository &repository, Oid const &id);

	Tree(Tree const &) = delete;
	Tree(Tree &&) = default;
	Tree &operator=(Tree const &) = delete;
	Tree &operator=(Tree &&) = default;

	~Tree() = default;

	Oid getId() const;

	void
	walk(std::function<bool(std::string const &)> const &callback) const;
};
}
}

#endif
