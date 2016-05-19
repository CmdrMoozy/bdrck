#ifndef bdrck_git_Config_HPP
#define bdrck_git_Config_HPP

#include <string>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;

class Config : public Wrapper<git_config, git_config_free>
{
private:
	typedef Wrapper<git_config, git_config_free> base_type;

public:
	Config();
	Config(Repository &repository);

	/**
	 * Creates a snapshot of the current state of this configuration
	 * object, which allows for looking up complex values.
	 *
	 * \return A new Config instance encapsulating the snapshot.
	 */
	Config snapshot();

	/**
	 * Get the configuration value associated with the given key, as a
	 * string.
	 *
	 * Note that like all get*() functions, this function must be called
	 * on a Config object returned by snapshot().
	 *
	 * Note that this configuration object will keep a copy of the
	 * C-string version of the returned value until it is destructed.
	 *
	 * \param key The configuration key to retrieve.
	 * \return The value associated with the given key, as a string.
	 */
	std::string getString(std::string const &key) const;

private:
	Config(git_config *config);
};
}
}

#endif
