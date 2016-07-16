#include "Oid.hpp"

#include <cstring>
#include <vector>

namespace
{
static_assert(sizeof(git_oid) == 20, "Git OID size matches expected size; "
                                     "otherwise, code changes may be needed.");
}

namespace bdrck
{
namespace git
{
Oid::Oid(git_oid const &o) : oid(o)
{
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

std::ostream &operator<<(std::ostream &out, Oid const &oid)
{
	out << oid.toString();
	return out;
}
}
}
