#include "Signature.hpp"

namespace bdrck
{
namespace git
{
Signature::Signature() : Signature(std::chrono::system_clock::now())
{
}

Signature::Signature(Repository &repository)
        : Signature(std::chrono::system_clock::now(), repository)
{
}

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
