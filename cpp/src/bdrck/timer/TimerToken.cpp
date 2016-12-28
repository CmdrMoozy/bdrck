#include "TimerToken.hpp"

namespace bdrck
{
namespace timer
{
TimerToken::TimerToken() : token(this, [](void *) {}), handle(token)
{
}
}
}
