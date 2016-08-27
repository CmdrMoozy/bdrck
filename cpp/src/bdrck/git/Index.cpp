#include "Index.hpp"

#include "bdrck/git/Repository.hpp"
#include "bdrck/git/StrArray.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
git_index *getRepositoryIndex(bdrck::git::Repository &repository)
{
	git_index *index = nullptr;
	bdrck::git::checkReturn(git_repository_index(&index, repository.get()));
	return index;
}
}

namespace bdrck
{
namespace git
{
Index::Index(Repository &repository) : base_type(getRepositoryIndex(repository))
{
}

std::size_t Index::getEntryCount() const
{
	return git_index_entrycount(get());
}

void Index::addAll(StrArray const &pathspec)
{
	checkReturn(
	        git_index_add_all(get(), &pathspec.get(), 0, nullptr, nullptr));
}

git_oid Index::writeTree()
{
	git_oid id;
	checkReturn(git_index_write_tree(&id, get()));
	return id;
}
}
}