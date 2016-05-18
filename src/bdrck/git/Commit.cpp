#include "Commit.hpp"

#include "bdrck/git/checkReturn.hpp"
#include "bdrck/git/Index.hpp"
#include "bdrck/git/Reference.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Signature.hpp"
#include "bdrck/git/StrArray.hpp"
#include "bdrck/git/Tree.hpp"

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
git_oid commitIndex(Repository &repository, Signature const &author,
                    Signature const &committer, std::string const &message,
                    std::string const &messageEncoding)
{
	Commit head(repository);
	git_commit const *parents[] = {head.get()};

	Index index(repository);
	Tree tree(repository, index.writeTree());

	git_oid id;
	checkReturn(git_commit_create(&id, repository.get(), "HEAD",
	                              &author.get(), &committer.get(),
	                              messageEncoding.c_str(), message.c_str(),
	                              tree.get(), 1, parents));
	return id;
}

git_oid commitAll(Repository &repository, Signature const &author,
                  Signature const &committer, std::string const &message,
                  std::string const &messageEncoding)
{
	Index index(repository);
	index.addAll({"."});
	return commitIndex(repository, author, committer, message,
	                   messageEncoding);
}

Commit::Commit(Repository &repository, git_oid const &id)
        : base_type(lookupCommit(repository, id))
{
}

Commit::Commit(Repository &repository)
        : Commit(repository, Reference(repository).getTarget())
{
}
}
}
