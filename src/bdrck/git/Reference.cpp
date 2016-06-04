#include "Reference.hpp"

#include <stdexcept>

#include "bdrck/git/Repository.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
git_reference *lookupReference(bdrck::git::Repository &repository,
                               std::string const &name)
{
	git_reference *reference = nullptr;
	bdrck::git::checkReturn(git_reference_lookup(
	        &reference, repository.get(), name.c_str()));
	return reference;
}
}

namespace bdrck
{
namespace git
{
Reference::Reference(Repository &repository, std::string const &name)
        : base_type(lookupReference(repository, name))
{
}

boost::optional<git_oid> Reference::getTarget() const
{
	git_oid const *oid = git_reference_target(get());
	if(oid == nullptr)
		return boost::none;
	else
		return *oid;
}

Reference Reference::resolve() const
{
	git_reference *reference = nullptr;
	checkReturn(git_reference_resolve(&reference, get()));
	return Reference(reference);
}

Reference::Reference(git_reference *reference) : base_type(reference)
{
}
}
}
