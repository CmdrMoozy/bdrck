#include "Tree.hpp"

#include "bdrck/fs/Util.hpp"
#include "bdrck/git/checkReturn.hpp"
#include "bdrck/git/Object.hpp"

namespace
{
git_tree *peelToTree(bdrck::git::Object const &object)
{
	git_object *peeled;
	bdrck::git::checkReturn(
	        git_object_peel(&peeled, object.get(), GIT_OBJ_TREE));
	return reinterpret_cast<git_tree *>(peeled);
}

int treeWalkCallback(char const *root, git_tree_entry const *entry,
                     void *payload)
{
	auto callback = static_cast<std::function<bool(std::string const &)> *>(
	        payload);
	return (*callback)(bdrck::fs::combinePaths(root,
	                                           git_tree_entry_name(entry)))
	               ? 0
	               : -1;
}
}

namespace bdrck
{
namespace git
{
Tree::Tree(Object const &object) : base_type(peelToTree(object))
{
}

Tree::~Tree()
{
}

void Tree::walk(std::function<bool(std::string const &)> callback) const
{
	checkReturn(git_tree_walk(get(), GIT_TREEWALK_PRE, treeWalkCallback,
	                          &callback));
}
}
}
