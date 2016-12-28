#ifndef bdrck_timer_TimerToken_HPP
#define bdrck_timer_TimerToken_HPP

#include <memory>
#include <utility>

namespace bdrck
{
namespace timer
{
/*!
 * \brief A token which can be used to cancel timed events.
 */
class TimerToken
{
public:
	TimerToken();

	TimerToken(TimerToken const &) = default;
	TimerToken(TimerToken &&) = default;
	TimerToken &operator=(TimerToken const &) = default;
	TimerToken &operator=(TimerToken &&) = default;

	~TimerToken() = default;

private:
	std::shared_ptr<void> token;

public:
	const std::weak_ptr<void> handle;
};
}
}

#endif
