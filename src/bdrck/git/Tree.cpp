#include "Tree.hpp"

#include <exception>

#include <boost/optional/optional.hpp>

#include "bdrck/fs/Util.hpp"
#include "bdrck/git/Commit.hpp"
#include "bdrck/git/Object.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
git_tree *peelToTree(bdrck::git::Object const &object)
{
	git_object *peeled;
	bdrck::git::checkReturn(
	        git_object_peel(&peeled, object.get(), GIT_OBJ_TREE));
	return reinterpret_cast<git_tree *>(peeled);
}

git_tree *commitToTree(bdrck::git::Commit const &commit)
{
	git_tree *tree;
	bdrck::git::checkReturn(git_commit_tree(&tree, commit.get()));
	return tree;
}

git_tree *lookupTree(bdrck::git::Repository &repository, git_oid const &id)
{
	git_tree *tree = nullptr;
	bdrck::git::checkReturn(git_tree_lookup(&tree, repository.get(), &id));
	return tree;
}

struct TreeWalkContext
{
	std::function<bool(std::string const &)> callback;
	int filemodeFilter;
	boost::optional<std::exception_ptr> error;

	TreeWalkContext(std::function<bool(std::string const &)> const &c,
	                int f)
	        : callback(c), filemodeFilter(f), error(boost::none)
	{
	}
};

int treeWalkCallback(char const *root, git_tree_entry const *entry,
                     void *payload)
{
	auto context = static_cast<TreeWalkContext *>(payload);

	git_filemode_t mode = git_tree_entry_filemode(entry);
	if((mode & context->filemodeFilter) == 0)
		return 0;

	try
	{
		return context->callback(bdrck::fs::combinePaths(
		               root, git_tree_entry_name(entry)))
		               ? 0
		               : -1;
	}
	catch(...)
	{
		context->error.emplace(std::current_exception());
		return -1;
	}
}
}

namespace bdrck
{
namespace git
{
Tree::Tree(Object const &obj) : base_type(peelToTree(obj))
{
}

Tree::Tree(Commit const &commit) : base_type(commitToTree(commit))
{
}

Tree::Tree(Repository &repository, Oid const &id)
        : base_type(lookupTree(repository, id.get()))
{
}

Oid Tree::getId() const
{
	return Oid(*git_tree_id(get()));
}

void Tree::walk(std::function<bool(std::string const &)> const &callback,
                int filemodeFilter) const
{
	TreeWalkContext context(callback, filemodeFilter);
	git_tree_walk(get(), GIT_TREEWALK_PRE, treeWalkCallback, &context);
	if(!!context.error)
		std::rethrow_exception(context.error.value());
}
}
}
