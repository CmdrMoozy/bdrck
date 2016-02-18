#include "PipeCast.hpp"

static_assert(sizeof(bdrck::process::NativePipe) <=
                      sizeof(bdrck::process::PipeDescriptor),
              "Native pipe representation must be compatible with "
              "PipeDescriptor type.");

namespace bdrck
{
namespace process
{
namespace pipe
{
NativePipe pipeCastToNative(PipeDescriptor p)
{
#ifdef _WIN32
	return reinterpret_cast<NativePipe>(p);
#else
	return static_cast<NativePipe>(p);
#endif
}

PipeDescriptor pipeCastFromNative(NativePipe p)
{
#ifdef _WIN32
	return reinterpret_cast<PipeDescriptor>(p);
#else
	return static_cast<PipeDescriptor>(p);
#endif
}
}
}
}
