#ifndef bdrck_params_parseAndExecute_HPP
#define bdrck_params_parseAndExecute_HPP

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
/*!
 * This function's purpose is very similar to parseAndExecuteCommand. This
 * function is a convenience wrapper for parsing command-line arguments for
 * binaries which do not have sub-commands (i.e., the binary only does one
 * thing).
 *
 * \param argc The number of command-line arguments.
 * \param argv The list of command-line arguments.
 * \param command The single command this binary supports.
 * \return The exit code; can be returned from main().
 */
int parseAndExecute(int argc, char const *const *argv, Command const &command);
}
}

#endif
