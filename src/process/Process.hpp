#ifndef bdrck_process_Process_HPP
#define bdrck_process_Process_HPP

#include <functional>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include <sys/types.h>

#include "bdrck/process/Terminal.hpp"

namespace bdrck
{
namespace process
{
namespace detail
{
struct Pipe
{
	int read;
	int write;

	explicit Pipe(int flags = 0);

	Pipe(Pipe const &) = default;
	Pipe(Pipe &&) = default;
	Pipe &operator=(Pipe const &) = default;
	Pipe &operator=(Pipe &&) = default;

	~Pipe() = default;
};
}

struct ProcessArguments
{
public:
	using ArgvSmartPointer =
	        std::unique_ptr<char, std::function<void(char *)>>;
	using ArgvContainer = std::vector<ArgvSmartPointer>;

	const std::string path;
	const std::vector<std::string> arguments;

private:
	const ArgvContainer argvContainer;
	const std::vector<char *> argvPointers;

public:
	char const *file;
	char *const *argv;

	ProcessArguments(std::string const &p,
	                 std::vector<std::string> const &a);
};

class Process
{
public:
	Process(std::string const &p, std::vector<std::string> const &a = {});

	Process(Process const &) = delete;
	Process(Process &&) = default;
	Process &operator=(Process const &) = delete;
	Process &operator=(Process &&) = default;

	~Process();

	int getPipe(terminal::StdStream stream) const;

	int wait();

private:
	ProcessArguments args;
	pid_t parent;
	pid_t child;
	std::map<terminal::StdStream, detail::Pipe> pipes;
};
}
}

#endif
