#include "TimerService.hpp"

#include <stdexcept>
#include <thread>

#include <boost/asio.hpp>
#include <boost/asio/basic_waitable_timer.hpp>

namespace
{
typedef boost::asio::basic_waitable_timer<std::chrono::steady_clock> Timer;

void runPeriodically(std::weak_ptr<boost::asio::io_service> weakService,
                     std::weak_ptr<void> cancellationHandle,
                     std::function<void()> const &function,
                     std::chrono::nanoseconds const &interval, bool repeat)
{
	auto service = weakService.lock();
	if(!service)
		return;

	auto timer = std::make_shared<Timer>(
	        *service, Timer::clock_type::now() + interval);
	auto handler =
	        [timer, weakService, cancellationHandle, function, interval,
	         repeat](boost::system::error_code const &error)
	{
		auto lock = cancellationHandle.lock();
		if(!lock)
			return;

		// If the timer didn't actually expire, just return.
		if(error)
			return;

		function();

		if(repeat)
		{
			runPeriodically(weakService, cancellationHandle,
			                function, interval, repeat);
		}
	};
	timer->async_wait(handler);
}
}

namespace bdrck
{
namespace timer
{
std::mutex TimerService::mutex;
boost::optional<TimerService> TimerService::singletonInstance;

namespace detail
{
struct TimerServiceImpl
{
	std::shared_ptr<boost::asio::io_service> service;
	boost::optional<boost::asio::io_service::work> work;
	std::thread thread;

	TimerServiceImpl();

	TimerServiceImpl(TimerServiceImpl const &) = delete;
	TimerServiceImpl(TimerServiceImpl &&) = default;
	TimerServiceImpl &operator=(TimerServiceImpl const &) = delete;
	TimerServiceImpl &operator=(TimerServiceImpl &&) = default;

	~TimerServiceImpl();
};

TimerServiceImpl::TimerServiceImpl()
        : service(std::make_shared<boost::asio::io_service>()),
          work(*service),
          thread([this]()
                 {
	                 for(;;)
	                 {
		                 try
		                 {
			                 service->run();
			                 break;
		                 }
		                 catch(std::exception const &)
		                 {
		                 }
	                 }
	         })
{
}

TimerServiceImpl::~TimerServiceImpl()
{
	work = boost::none;
	service->stop();

	try
	{
		thread.join();
	}
	catch(std::exception const &)
	{
	}
}
}

TimerServiceInstance::TimerServiceInstance()
{
	std::lock_guard<std::mutex> lock(TimerService::mutex);
	if(!!TimerService::singletonInstance)
	{
		throw std::runtime_error(
		        "TimerService singleton already initialized.");
	}

	TimerService service;
	TimerService::singletonInstance = std::move(service);
}

TimerServiceInstance::~TimerServiceInstance()
{
	std::lock_guard<std::mutex> lock(TimerService::mutex);
	TimerService::singletonInstance = boost::none;
}

TimerService &TimerService::instance()
{
	std::lock_guard<std::mutex> lock(mutex);
	if(!singletonInstance)
	{
		throw std::runtime_error(
		        "TimerService singleton not initialized.");
	}
	return *singletonInstance;
}

TimerService::~TimerService()
{
}

TimerService::TimerService()
        : impl(std::make_unique<detail::TimerServiceImpl>())
{
}

TimerToken TimerService::runOnceInImpl(std::function<void()> const &function,
                                       std::chrono::nanoseconds const &delay)
{
	TimerToken token;
	runPeriodically(impl->service, token.handle, function, delay, false);
	return token;
}

TimerToken TimerService::runEveryImpl(std::function<void()> const &function,
                                      std::chrono::nanoseconds const &interval)
{
	TimerToken token;
	runPeriodically(impl->service, token.handle, function, interval, true);
	return token;
}
}
}
