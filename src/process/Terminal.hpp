#ifndef bdrck_process_Terminal_HPP
#define bdrck_process_Terminal_HPP

namespace bdrck
{
namespace process
{
namespace terminal
{
enum class StdStream
{
	In,
	Out,
	Err
};

int streamFD(StdStream stream);
bool isInteractiveTerminal(StdStream stream);
}
}
}

#endif
