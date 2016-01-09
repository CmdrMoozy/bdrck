#include "Repository.hpp"

#include <stdexcept>

#include "bdrck/fs/Util.hpp"
#include "bdrck/git/Buffer.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
std::string discover(std::string const &p)
{
	bdrck::git::Buffer buffer;
	bdrck::git::checkReturn(
	        git_repository_discover(buffer.get(), p.c_str(), 0, nullptr));
	return std::string(buffer.begin(), buffer.end());
}

std::string getRepositoryConstructPath(std::string const &p,
                                       bdrck::git::RepositoryCreateMode c,
                                       bool ab)
{
	try
	{
		return discover(p);
	}
	catch(...)
	{
		if(c == bdrck::git::RepositoryCreateMode::NoCreate)
			throw;
		if(!ab && (c == bdrck::git::RepositoryCreateMode::CreateBare))
			throw;

		bdrck::fs::createPath(p);
		git_repository *repo;
		bdrck::git::checkReturn(git_repository_init(
		        &repo, p.c_str(),
		        c == bdrck::git::RepositoryCreateMode::CreateNormal
		                ? 0
		                : 1));
		git_repository_free(repo);
		return p;
	}
}
}

namespace bdrck
{
namespace git
{
Repository::Repository(std::string const &p, RepositoryCreateMode c, bool ab)
        : base_type(git_repository_open,
                    getRepositoryConstructPath(p, c, ab).c_str())
{
}

Repository::~Repository()
{
}

std::string Repository::getWorkDirectoryPath() const
{
	char const *path = git_repository_workdir(get());
	if(path == nullptr)
	{
		throw std::runtime_error(
		        "This repository has no work directory.");
	}
	return path;
}
}
}
