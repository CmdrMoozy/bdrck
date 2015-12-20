#ifndef bdrck_params_detail_Help_HPP
#define bdrck_params_detail_Help_HPP

#include <set>
#include <string>

#include "bdrck/params/Command.hpp"

namespace bdrck
{
namespace params
{
namespace detail
{
/*!
 * Print help information for the given program.
 *
 * \param program The name of the binary being executed.
 * \param commands The set of commands this binary supports.
 */
void printProgramHelp(std::string const &program,
                      std::set<bdrck::params::Command> const &commands);

/*!
 * Print help information for the given command. If printCommandName is false,
 * then the name of this particular command will be omitted, and the arguments
 * will be displayed as if they were just program arguments.
 *
 * \param program The name of the binary being executed.
 * \param command The command to print help for.
 * \param printCommandName Whether or not to print the command name.
 */
void printCommandHelp(std::string const &program,
                      bdrck::params::Command const &command,
                      bool printCommandName = true);
}
}
}

#endif
