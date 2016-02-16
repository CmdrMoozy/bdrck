#ifndef bdrck_git_Tree_HPP
#define bdrck_git_Tree_HPP

#include <functional>
#include <string>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Object;

class Tree : public Wrapper<git_tree, git_tree_free>
{
private:
	typedef Wrapper<git_tree, git_tree_free> base_type;

public:
	Tree(Object const &object);

	~Tree() = default;

	void
	walk(std::function<bool(std::string const &)> const &callback) const;
};
}
}

#endif