#ifndef bdrck_git_Commit_HPP
#define bdrck_git_Commit_HPP

#include <string>

#include <git2.h>

#include "bdrck/git/Signature.hpp"
#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;

constexpr char const *DEFAULT_MESSAGE_ENCODING = "UTF-8";

/**
 * This is a convenience function which will create a new commit containing
 * any files already in the index when this function is called.
 */
git_oid
commitIndex(Repository &repository, std::string const &message,
            Signature const &author = Signature(),
            Signature const &committer = Signature(),
            std::string const &messageEncoding = DEFAULT_MESSAGE_ENCODING);

/**
 * This is a convenience function which will create a new commit containing
 * any uncommitted files present when this function is called (this is
 * essentially the same as "git add . && git commit").
 */
git_oid
commitAll(Repository &repository, std::string const &message,
          Signature const &author = Signature(),
          Signature const &committer = Signature(),
          std::string const &messageEncoding = DEFAULT_MESSAGE_ENCODING);

class Commit : public Wrapper<git_commit, git_commit_free>
{
private:
	typedef Wrapper<git_commit, git_commit_free> base_type;

public:
	Commit(Repository &repository, git_oid const &id);

	/**
	 * Construct a new Commit referring to the given repository's current
	 * HEAD commit.
	 */
	Commit(Repository &repository);
};
}
}

#endif
