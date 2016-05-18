#ifndef bdrck_git_Reference_HPP
#define bdrck_git_Reference_HPP

#include <string>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;

class Reference : public Wrapper<git_reference, git_reference_free>
{
private:
	typedef Wrapper<git_reference, git_reference_free> base_type;

public:
	Reference(Repository &repository, std::string const &name = "HEAD");

	git_oid getTarget() const;
	Reference resolve() const;

private:
	Reference(git_reference *reference);
};
}
}

#endif
