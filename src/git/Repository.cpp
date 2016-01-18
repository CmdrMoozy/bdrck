#include "Repository.hpp"

#include <stdexcept>

#include "bdrck/fs/Util.hpp"
#include "bdrck/git/Buffer.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
std::string getRepositoryConstructPath(std::string const &p,
                                       bdrck::git::RepositoryCreateMode c,
                                       bool ab)
{
	auto path = bdrck::git::discoverRepository(p);

	if(!path)
	{
		if((c == bdrck::git::RepositoryCreateMode::NoCreate) ||
		   (!ab && (c == bdrck::git::RepositoryCreateMode::CreateBare)))
		{
			throw std::runtime_error("Repository doesn't exist and "
			                         "will not be created.");
		}

		path.emplace(p);
		bdrck::fs::createPath(*path);
		git_repository *repo;
		bdrck::git::checkReturn(git_repository_init(
		        &repo, p.c_str(),
		        c == bdrck::git::RepositoryCreateMode::CreateNormal
		                ? 0
		                : 1));
		git_repository_free(repo);
	}

	return *path;
}
}

namespace bdrck
{
namespace git
{
std::experimental::optional<std::string>
discoverRepository(std::string const &path, bool acrossFilesystems) noexcept
{
	try
	{
		bdrck::git::Buffer buffer;
		int ret = git_repository_discover(buffer.get(), path.c_str(),
		                                  acrossFilesystems ? 1 : 0,
		                                  nullptr);
		if(ret == 0)
			return std::string(buffer.begin(), buffer.end());
	}
	catch(...)
	{
	}

	return std::experimental::nullopt;
}

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
