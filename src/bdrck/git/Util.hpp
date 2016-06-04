#ifndef bdrck_git_Util_HPP
#define bdrck_git_Util_HPP

#include <chrono>
#include <stdexcept>
#include <string>

#include <boost/date_time/c_local_time_adjustor.hpp>
#include <boost/date_time/posix_time/posix_time.hpp>

#include <git2.h>

namespace bdrck
{
namespace git
{
/**
 * Convert a standard time point to a git_time_t, which is an integer number
 * of seconds since the epoch.
 *
 * \param when The time point to convert.
 * \return The time point as a number of seconds since the epoch.
 */
template <typename Clock, typename Duration = typename Clock::duration>
git_time_t toGitTimestamp(std::chrono::time_point<Clock, Duration> const &when)
{
	return static_cast<git_time_t>(
	        std::chrono::duration_cast<std::chrono::seconds>(
	                when.time_since_epoch())
	                .count());
}

/**
 * Convert a standard time point to a git_time structure, using the system's
 * current timezone offset.
 *
 * \param when The time point to convert.
 * \return The git_time structure from the given time point.
 */
template <typename Clock, typename Duration = typename Clock::duration>
git_time toGitTime(std::chrono::time_point<Clock, Duration> const &when)
{
	const auto utcNow = boost::posix_time::second_clock::universal_time();
	const auto now = boost::date_time::c_local_adjustor<
	        boost::posix_time::ptime>::utc_to_local(utcNow);
	boost::posix_time::time_duration offset = now - utcNow;

	return {toGitTimestamp(when), static_cast<int>(offset.total_seconds())};
}

std::string oidToString(git_oid const &oid);
}
}

#endif
