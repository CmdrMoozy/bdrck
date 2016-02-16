#include "ScopeExit.hpp"

namespace bdrck
{
namespace util
{
ScopeExit::ScopeExit(std::function<void()> f) : function(f)
{
}

ScopeExit::~ScopeExit()
{
	function();
}
}
}
