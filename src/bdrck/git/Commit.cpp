#include "Commit.hpp"

#include "bdrck/git/Index.hpp"
#include "bdrck/git/Object.hpp"
#include "bdrck/git/Reference.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Signature.hpp"
#include "bdrck/git/StrArray.hpp"
#include "bdrck/git/Tree.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
git_commit *lookupCommit(bdrck::git::Repository &repository, git_oid const &id)
{
	git_commit *commit = nullptr;
	bdrck::git::checkReturn(
	        git_commit_lookup(&commit, repository.get(), &id));
	return commit;
}
}

namespace bdrck
{
namespace git
{
git_oid commitTree(Repository &repository, std::string const &message,
                   Tree const &tree, Signature const &author,
                   Signature const &committer,
                   std::string const &messageEncoding)
{
	git_commit const *parents[] = {nullptr};
	Reference headRef(repository);
	auto headId = headRef.getTarget();
	boost::optional<Commit> head;
	if(!!headId)
	{
		head.emplace(repository, *headId);
		parents[0] = head->get();
	}

	git_oid id;
	checkReturn(git_commit_create(
	        &id, repository.get(), "HEAD", &author.get(), &committer.get(),
	        messageEncoding.c_str(), message.c_str(), tree.get(),
	        parents[0] == nullptr ? 0 : 1, parents));

	Object headObj("HEAD", repository);
	checkReturn(git_reset(repository.get(), headObj.get(), GIT_RESET_MIXED,
	                      nullptr));

	return id;
}

git_oid commitIndex(Repository &repository, std::string const &message,
                    Signature const &author, Signature const &committer,
                    std::string const &messageEncoding)
{
	Index index(repository);
	Tree tree(repository, index.writeTree());
	return commitTree(repository, message, tree, author, committer,
	                  messageEncoding);
}

git_oid commitAll(Repository &repository, std::string const &message,
                  Signature const &author, Signature const &committer,
                  std::string const &messageEncoding)
{
	Index index(repository);
	index.addAll({"."});
	return commitIndex(repository, message, author, committer,
	                   messageEncoding);
}

Commit::Commit(Repository &repository, git_oid const &id)
        : base_type(lookupCommit(repository, id))
{
}

Commit::Commit(Repository &repository)
        : Commit(repository, *Reference(repository).getTarget())
{
}
}
}
