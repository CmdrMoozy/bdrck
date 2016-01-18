#include "Tree.hpp"

#include <exception>
#include <experimental/optional>

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

struct TreeWalkContext
{
	std::function<bool(std::string const &)> callback;
	std::experimental::optional<std::exception_ptr> error;

	TreeWalkContext(std::function<bool(std::string const &)> const &c)
	        : callback(c), error(std::experimental::nullopt)
	{
	}
};

int treeWalkCallback(char const *root, git_tree_entry const *entry,
                     void *payload)
{
	auto context = static_cast<TreeWalkContext *>(payload);

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
Tree::Tree(Object const &object) : base_type(peelToTree(object))
{
}

Tree::~Tree()
{
}

void Tree::walk(std::function<bool(std::string const &)> const &callback) const
{
	TreeWalkContext context(callback);
	git_tree_walk(get(), GIT_TREEWALK_PRE, treeWalkCallback, &context);
	if(!!context.error)
		std::rethrow_exception(context.error.value());
}
}
}
