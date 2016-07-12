#ifndef bdrck_git_Signature_HPP
#define bdrck_git_Signature_HPP

#include <chrono>
#include <string>
#include <vector>

#include <boost/optional/optional.hpp>

#include <git2.h>

#include "bdrck/git/Config.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Util.hpp"

namespace bdrck
{
namespace git
{
class Signature
{
public:
	template <typename Clock, typename Duration = typename Clock::duration>
	Signature(std::string const &n, std::string const &e,
	          std::chrono::time_point<Clock, Duration> const &when);

	/**
	 * Construct a signature using user.name and user.email from the given
	 * repository's Git configuration, and the given time point.
	 *
	 * \param repository The repository to load Git configuration from.
	 */
	template <typename Clock, typename Duration = typename Clock::duration>
	Signature(std::chrono::time_point<Clock, Duration> const &when,
	          Repository &repository);

	/**
	 * Construct a signature using user.name and user.email from Git's
	 * configuration, and the given time point.
	 */
	template <typename Clock, typename Duration = typename Clock::duration>
	Signature(std::chrono::time_point<Clock, Duration> const &when);

	/**
	 * Construct the default signature, which uses user.name and
	 * user.email from the given repository's Git configuration and the
	 * current time.
	 *
	 * \param repository The repository to load Git configuration from.
	 */
	Signature(Repository &repository);

	/**
	 * Construct the default signature, which uses user.name and
	 * user.email from Git's configuration and the current time.
	 */
	Signature();

	Signature(Signature const &) = default;
	Signature(Signature &&) = default;
	Signature &operator=(Signature const &) = default;
	Signature &operator=(Signature &&) = default;

	~Signature() = default;

	git_signature &get();
	git_signature const &get() const;

private:
	std::vector<char> name;
	std::vector<char> email;
	git_signature signature;

	template <typename Clock, typename Duration = typename Clock::duration>
	Signature(std::chrono::time_point<Clock, Duration> const &when,
	          Config const &config);
};

template <typename Clock, typename Duration>
Signature::Signature(std::string const &n, std::string const &e,
                     std::chrono::time_point<Clock, Duration> const &when)
        : name(n.c_str(), n.c_str() + n.length() + 1),
          email(e.c_str(), e.c_str() + e.length() + 1),
          signature({name.data(), email.data(), toGitTime(when)})
{
}

template <typename Clock, typename Duration>
Signature::Signature(std::chrono::time_point<Clock, Duration> const &when,
                     Repository &repository)
        : Signature(when, Config(repository).snapshot())
{
}

template <typename Clock, typename Duration>
Signature::Signature(std::chrono::time_point<Clock, Duration> const &when)
        : Signature(when, Config().snapshot())
{
}

template <typename Clock, typename Duration>
Signature::Signature(std::chrono::time_point<Clock, Duration> const &when,
                     Config const &config)
        : Signature(config.getString("user.name"),
                    config.getString("user.email"), when)
{
}
}
}

#endif
