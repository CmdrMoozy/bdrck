#include "Object.hpp"

#include "bdrck/git/checkReturn.hpp"
#include "bdrck/git/Repository.hpp"

namespace
{
git_object *parseRevspec(std::string const &revspec,
                         bdrck::git::Repository &repository)
{
	git_object *object;
	bdrck::git::checkReturn(git_revparse_single(&object, repository.get(),
	                                            revspec.c_str()));
	return object;
}
}

namespace bdrck
{
namespace git
{
Object::Object(std::string const &revspec, Repository &repository)
        : base_type(parseRevspec(revspec, repository))
{
}

Object::~Object()
{
}
}
}
