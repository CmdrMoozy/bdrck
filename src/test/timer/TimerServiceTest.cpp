#include <catch/catch.hpp>

#include <chrono>
#include <cmath>
#include <condition_variable>
#include <mutex>
#include <thread>

#include "bdrck/timer/TimerService.hpp"

TEST_CASE("Test timer single runs", "[TimerService]")
{
	bdrck::timer::TimerServiceInstance instance;
	constexpr int TEST_DELAY_MS = 100;
	constexpr int TEST_DELAY_ERROR_TOLERANCE_MS = 5;

	std::mutex mutex;
	std::condition_variable condition;

	std::chrono::high_resolution_clock::time_point start, end;
	auto function = [&mutex, &condition, &end]()
	{
		std::lock_guard<std::mutex> lock(mutex);
		end = std::chrono::high_resolution_clock::now();
		condition.notify_one();
	};

	std::unique_lock<std::mutex> lock(mutex);
	start = std::chrono::high_resolution_clock::now();
	auto token = bdrck::timer::TimerService::instance().runOnceIn(
	        function, std::chrono::milliseconds(TEST_DELAY_MS));
	condition.wait(lock);

	auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
	                       end - start).count();
	CHECK(std::abs(elapsed - TEST_DELAY_MS) <=
	      TEST_DELAY_ERROR_TOLERANCE_MS);
}

TEST_CASE("Test timer repeated runs", "[TimerService]")
{
	bdrck::timer::TimerServiceInstance instance;
	constexpr int TEST_INTERVAL_MS = 100;
	constexpr int TEST_DURATION_MS = 300;
	constexpr int EXPECTED_TIMES_EXECUTED = 3;
	constexpr int EXPECTED_TIMES_EXECUTED_ERROR_TOLERANCE = 1;

	int timesExecuted = 0;
	auto function = [&timesExecuted]()
	{
		++timesExecuted;
	};

	{
		auto token = bdrck::timer::TimerService::instance().runEvery(
		        function, std::chrono::milliseconds(TEST_INTERVAL_MS));
		std::this_thread::sleep_for(
		        std::chrono::milliseconds(TEST_DURATION_MS));
	}

	CHECK(std::abs(timesExecuted - EXPECTED_TIMES_EXECUTED) <=
	      EXPECTED_TIMES_EXECUTED_ERROR_TOLERANCE);
}
