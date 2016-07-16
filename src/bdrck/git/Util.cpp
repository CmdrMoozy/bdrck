#include "Util.hpp"

#include <stdexcept>
#include <vector>

#include "bdrck/git/checkReturn.hpp"
#include "bdrck/util/ScopeExit.hpp"

namespace bdrck
{
namespace git
{
boost::optional<Oid> revspecToOid(std::string const &revspec,
                                  Repository &repository)
{
	git_revspec rs;
	int ret = git_revparse(&rs, repository.get(), revspec.c_str());
	if(ret == GIT_ENOTFOUND)
		return boost::none;
	checkReturn(ret);
	bdrck::util::ScopeExit revspecCleanup([&rs]() {
		git_object_free(rs.from);
		git_object_free(rs.to);
	});

	if(rs.flags & GIT_REVPARSE_SINGLE)
		return Oid(*git_object_id(rs.from));
	else
		throw std::runtime_error("Error parsing non-single revspec.");
}

std::string oidToString(git_oid const &oid)
{
	std::vector<char> buffer(40);
	git_oid_fmt(buffer.data(), &oid);
	return std::string(buffer.begin(), buffer.end());
}
}
}
