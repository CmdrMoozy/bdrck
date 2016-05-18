#ifndef bdrck_git_Signature_HPP
#define bdrck_git_Signature_HPP

#include <chrono>
#include <string>

#include <git2.h>

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

	git_signature &get();
	git_signature const &get() const;

private:
	std::string name;
	std::string email;
	git_signature signature;
};

template <typename Clock, typename Duration>
Signature::Signature(std::string const &n, std::string const &e,
                     std::chrono::time_point<Clock, Duration> const &when)
        : name(n),
          email(e),
          signature({name.c_str(), email.c_str(), toGitTime(when)})
{
}
}
}

#endif
