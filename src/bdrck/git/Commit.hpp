#ifndef bdrck_git_Commit_HPP
#define bdrck_git_Commit_HPP

#include <string>

#include <boost/optional/optional.hpp>

#include <git2.h>

#include "bdrck/git/Oid.hpp"
#include "bdrck/git/Signature.hpp"
#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;
class Tree;

constexpr char const *DEFAULT_MESSAGE_ENCODING = "UTF-8";

/**
 * This is a convenience function which will create a new commit with the given
 * tree.
 *
 * If the given tree is the empty tree, or if it is the same tree the HEAD
 * commit references, then the new commit would be empty, and thus it is not
 * created and boost::none is returned instead.
 */
boost::optional<Oid>
commitTree(Repository &repository, std::string const &message, Tree const &tree,
           Signature const &author = Signature(),
           Signature const &committer = Signature(),
           std::string const &messageEncoding = DEFAULT_MESSAGE_ENCODING);

/**
 * This is a convenience function which will create a new commit containing
 * any files already in the index when this function is called.
 *
 * If the repository's index is entry, no commit is created, and boost::none
 * is returned instead.
 */
boost::optional<Oid>
commitIndex(Repository &repository, std::string const &message,
            Signature const &author = Signature(),
            Signature const &committer = Signature(),
            std::string const &messageEncoding = DEFAULT_MESSAGE_ENCODING);

/**
 * This is a convenience function which will create a new commit containing
 * any uncommitted files present when this function is called (this is
 * essentially the same as "git add . && git commit").
 *
 * Note that, if there are no changes to commit, no commit will be created, in
 * which case boost::none is returned instead of a valid OID.
 */
boost::optional<Oid>
commitAll(Repository &repository, std::string const &message,
          Signature const &author = Signature(),
          Signature const &committer = Signature(),
          std::string const &messageEncoding = DEFAULT_MESSAGE_ENCODING);

class Commit : public Wrapper<git_commit, git_commit_free>
{
private:
	typedef Wrapper<git_commit, git_commit_free> base_type;

public:
	Commit(Repository &repository, Oid const &id);

	Commit(Commit const &) = delete;
	Commit(Commit &&) = default;
	Commit &operator=(Commit const &) = delete;
	Commit &operator=(Commit &&) = default;

	/**
	 * Construct a new Commit referring to the given repository's current
	 * HEAD commit.
	 */
	Commit(Repository &repository);
};
}
}

#endif
