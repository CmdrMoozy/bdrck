#ifndef bdrck_fs_Util_HPP
#define bdrck_fs_Util_HPP

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

#include <boost/optional/optional.hpp>

namespace bdrck
{
namespace fs
{
typedef std::chrono::time_point<std::chrono::high_resolution_clock>
        FilesystemTime;

void lastWriteTime(std::string const &p, FilesystemTime const &t);
}
}

#endif
