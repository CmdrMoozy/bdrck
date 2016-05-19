#include "Config.hpp"

#include "bdrck/git/checkReturn.hpp"
#include "bdrck/git/Repository.hpp"

namespace
{
git_config *openConfig()
{
	git_config *config = nullptr;
	bdrck::git::checkReturn(git_config_open_default(&config));
	return config;
}

git_config *openRepositoryConfig(bdrck::git::Repository &repository)
{
	git_config *config = nullptr;
	bdrck::git::checkReturn(
	        git_repository_config(&config, repository.get()));
	return config;
}
}

namespace bdrck
{
namespace git
{
Config::Config() : base_type(openConfig())
{
}

Config::Config(Repository &repository)
        : base_type(openRepositoryConfig(repository))
{
}

Config Config::snapshot()
{
	git_config *config = nullptr;
	git_config_snapshot(&config, get());
	return Config(config);
}

std::string Config::getString(std::string const &key) const
{
	char const *cstr;
	checkReturn(git_config_get_string(&cstr, get(), key.c_str()));
	return std::string(cstr);
}

Config::Config(git_config *config) : base_type(config)
{
}
}
}
