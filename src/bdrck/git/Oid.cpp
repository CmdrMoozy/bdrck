#include "Oid.hpp"

#include <cstring>
#include <vector>

#include "bdrck/git/checkReturn.hpp"

namespace
{
static_assert(sizeof(git_oid) == 20, "Git OID size matches expected size; "
                                     "otherwise, code changes may be needed.");

constexpr char const *EMPTY_TREE_OID =
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
}

namespace bdrck
{
namespace git
{
Oid::Oid(git_oid const &o) : oid(o)
{
}

Oid::Oid(std::string const &o) : oid()
{
	checkReturn(git_oid_fromstr(&oid, o.c_str()));
}

int Oid::compare(Oid const &o) const
{
	return std::memcmp(&oid.id[0], &o.oid.id[0], sizeof(git_oid));
}

bool Oid::operator==(Oid const &o) const
{
	return compare(o) == 0;
}

bool Oid::operator!=(Oid const &o) const
{
	return !(*this == o);
}

bool Oid::operator<(Oid const &o) const
{
	return compare(o) < 0;
}

bool Oid::operator<=(Oid const &o) const
{
	return compare(o) <= 0;
}

bool Oid::operator>(Oid const &o) const
{
	return compare(o) > 0;
}

bool Oid::operator>=(Oid const &o) const
{
	return compare(o) >= 0;
}

git_oid const &Oid::get() const
{
	return oid;
}

std::string Oid::toString() const
{
	std::vector<char> data(sizeof(git_oid) * 2);
	git_oid_fmt(data.data(), &oid);
	return std::string(data.data(), sizeof(git_oid) * 2);
}

Oid getEmptyTreeOid()
{
	static const Oid emptyTreeOid{std::string(EMPTY_TREE_OID)};
	return emptyTreeOid;
}

std::ostream &operator<<(std::ostream &out, Oid const &oid)
{
	out << oid.toString();
	return out;
}
}
}
