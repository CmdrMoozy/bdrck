#ifndef bdrck_git_Object_HPP
#define bdrck_git_Object_HPP

#include <string>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;

class Object : public Wrapper<git_object, git_object_free>
{
private:
	typedef Wrapper<git_object, git_object_free> base_type;

public:
	Object(std::string const &revspec, Repository &repository);

	Object(Object const &) = delete;
	Object(Object &&) = default;
	Object &operator=(Object const &) = delete;
	Object &operator=(Object &&) = default;

	~Object() = default;

	git_oid getId() const;
};
}
}

#endif
