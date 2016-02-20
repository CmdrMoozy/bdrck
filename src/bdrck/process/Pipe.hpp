#ifndef bdrck_process_Pipe_HPP
#define bdrck_process_Pipe_HPP

#include <cstdint>
#include <map>
#include <memory>
#include <string>

namespace bdrck
{
namespace process
{
enum class StdStream
{
	STDIN,
	STDOUT,
	STDERR
};

/*!
 * The type used for pipe descriptors on all platforms. Values of this type
 * can be cast to the current platform's pipe descriptor type safely.
 */
typedef int64_t PipeDescriptor;

namespace detail
{
struct PipeImpl;
}

enum class PipeSide
{
	READ,
	WRITE
};

/*!
 * A class which encapsulates the platform-dependent code needed to create
 * pipes. NOTE: This structure's destructor does NOT close the pipe - it is
 * up to the user of this class to do this. Because pipe descriptors can be
 * reused on some platforms, it is impossible for this class to track whether
 * or not a pipe has already been closed (consider, for instance, UNIX file
 * descriptor reuse).
 */
class Pipe
{
public:
	Pipe();
#ifndef _WIN32
	explicit Pipe(int flags);
#endif

	Pipe(Pipe const &o);
	Pipe &operator=(Pipe const &o);

	Pipe(Pipe &&) = default;
	Pipe &operator=(Pipe &&) = default;

	~Pipe();

	PipeDescriptor get(PipeSide side) const;
	void set(PipeSide side, PipeDescriptor descriptor);

private:
	std::unique_ptr<detail::PipeImpl> impl;
};

typedef std::map<StdStream, Pipe> StandardStreamPipes;

namespace pipe
{
PipeDescriptor getStreamPipe(StdStream stream);
bool isInteractiveTerminal(PipeDescriptor pipe);

void openPipes(StandardStreamPipes &pipes);

std::string read(PipeDescriptor const &pipe, std::size_t count);
std::string read(Pipe const &pipe, PipeSide side, std::size_t count);
std::string readAll(PipeDescriptor const &pipe);
std::string readAll(Pipe const &pipe, PipeSide side);
std::size_t write(PipeDescriptor const &pipe, void const *buffer,
                  std::size_t size);
std::size_t write(Pipe const &pipe, PipeSide side, void const *buffer,
                  std::size_t size);

void close(PipeDescriptor const &pipe);
void close(Pipe const &pipe, PipeSide side);
void closeParentSide(StandardStreamPipes const &pipes);
void closeChildSide(StandardStreamPipes const &pipes);
}
}
}

#endif
