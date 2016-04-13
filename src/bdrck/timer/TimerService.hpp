#ifndef bdrck_timer_TimerService_HPP
#define bdrck_timer_TimerService_HPP

#include <chrono>
#include <functional>
#include <memory>
#include <mutex>

#include <boost/optional/optional.hpp>

#include "bdrck/timer/TimerToken.hpp"

namespace bdrck
{
namespace timer
{
namespace detail
{
struct TimerServiceImpl;
}

/*!
 * \brief A RAII-style handle which initializes the TimerService singleton.
 */
class TimerServiceInstance
{
public:
	TimerServiceInstance();

	TimerServiceInstance(TimerServiceInstance const &) = delete;
	TimerServiceInstance(TimerServiceInstance &&) = default;
	TimerServiceInstance &operator=(TimerServiceInstance const &) = delete;
	TimerServiceInstance &operator=(TimerServiceInstance &&) = default;

	~TimerServiceInstance();
};

/*!
 * \brief A singleton for executing code periodically or after some delay.
 */
class TimerService
{
public:
	static TimerService &instance();

	TimerService(TimerService const &) = delete;
	TimerService(TimerService &&) = default;
	TimerService &operator=(TimerService const &) = delete;
	TimerService &operator=(TimerService &&) = default;

	~TimerService();

	/*!
	 * Run a callback once after a particular delay period.
	 *
	 * The returned token must be kept alive until after the callback
	 * is executed. If the token is destructed before this happens, the
	 * execution of the callback will be cancelled.
	 */
	template <typename Rep, typename Period>
	TimerToken runOnceIn(std::function<void()> const &function,
	                     std::chrono::duration<Rep, Period> const &delay);

	/*!
	 * Run a callback once at a particular time. If the given time point
	 * is already in the past, the function is executed immediately.
	 *
	 * The returned token must be kept alive until after the callback
	 * is executed. If the token is destructed before this happens, the
	 * execution of the callback will be cancelled.
	 */
	template <typename Clock, typename Duration>
	TimerToken
	runOnceAt(std::function<void()> const &function,
	          std::chrono::time_point<Clock, Duration> const &time);

	template <typename Rep, typename Period>
	TimerToken runEvery(std::function<void()> const &function,
	                    std::chrono::duration<Rep, Period> const &interval);

private:
	friend class TimerServiceInstance;

	static std::mutex mutex;
	static boost::optional<TimerService> singletonInstance;

	std::unique_ptr<detail::TimerServiceImpl> impl;

	TimerService();

	TimerToken runOnceInImpl(std::function<void()> const &function,
	                         std::chrono::nanoseconds const &delay);
	TimerToken runEveryImpl(std::function<void()> const &function,
	                        std::chrono::nanoseconds const &interval);
};

template <typename Rep, typename Period>
TimerToken
TimerService::runOnceIn(std::function<void()> const &function,
                        std::chrono::duration<Rep, Period> const &delay)
{
	return runOnceInImpl(
	        function,
	        std::chrono::duration_cast<std::chrono::nanoseconds>(delay));
}

template <typename Clock, typename Duration>
TimerToken
TimerService::runOnceAt(std::function<void()> const &function,
                        std::chrono::time_point<Clock, Duration> const &time)
{
	return runOnceIn(function, time - Clock::now());
}

template <typename Rep, typename Period>
TimerToken
TimerService::runEvery(std::function<void()> const &function,
                       std::chrono::duration<Rep, Period> const &interval)
{
	return runEveryImpl(
	        function,
	        std::chrono::duration_cast<std::chrono::nanoseconds>(interval));
}
}
}

#endif
