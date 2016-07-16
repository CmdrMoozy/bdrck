#ifndef bdrck_git_Oid_HPP
#define bdrck_git_Oid_HPP

#include <string>

#include <git2.h>

namespace bdrck
{
namespace git
{
class Oid
{
public:
	Oid(git_oid const &o);

	Oid(Oid const &) = default;
	Oid(Oid &&) = default;
	Oid &operator=(Oid const &) = default;
	Oid &operator=(Oid &&) = default;

	~Oid() = default;

	int compare(Oid const &o) const;
	bool operator==(Oid const &o) const;
	bool operator!=(Oid const &o) const;
	bool operator<(Oid const &o) const;
	bool operator<=(Oid const &o) const;
	bool operator>(Oid const &o) const;
	bool operator>=(Oid const &o) const;

	git_oid const &get() const;

	std::string toString() const;

private:
	git_oid oid;
};

std::ostream &operator<<(std::ostream &out, Oid const &oid);
}
}

#endif
