#ifndef bdrck_process_PipeCast_HPP
#define bdrck_process_PipeCast_HPP

#include "bdrck/process/Pipe.hpp"

// NOTE: Due to the fact that this header includes Windows.h, it should not be
// included from other header files (generally). Use these functions in an
// implementation file.
#ifdef _WIN32
#include <Windows.h>
#else
#include <fcntl.h>
#include <unistd.h>
#endif

namespace bdrck
{
namespace process
{
#ifdef _WIN32
typedef HANDLE NativePipe;
constexpr NativePipe INVALID_PIPE_VALUE = INVALID_HANDLE_VALUE;
#else
typedef int NativePipe;
constexpr NativePipe INVALID_PIPE_VALUE = -1;
#endif

namespace pipe
{
NativePipe pipeCastToNative(PipeDescriptor p);
PipeDescriptor pipeCastFromNative(NativePipe p);
}
}
}

#endif
