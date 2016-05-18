#include "Signature.hpp"

namespace bdrck
{
namespace git
{
git_signature &Signature::get()
{
	return signature;
}

git_signature const &Signature::get() const
{
	return signature;
}
}
}
