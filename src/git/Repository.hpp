#ifndef bdrck_git_Repository_HPP
#define bdrck_git_Repository_HPP

#include <string>
#include <experimental/optional>

#include <git2.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
constexpr char const *GIT_EMPTY_TREE_OID =
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

std::experimental::optional<std::string>
discoverRepository(std::string const &path,
                   bool acrossFilesystems = false) noexcept;

enum class RepositoryCreateMode
{
	NoCreate,
	CreateNormal,
	CreateBare
};

class Repository : public Wrapper<git_repository, git_repository_free>
{
private:
	typedef Wrapper<git_repository, git_repository_free> base_type;

public:
	/*!
	 * \param p The path to the repository to open.
	 * \param c The creation mode to use if it doesn't already exist.
	 * \param ab Whether or not to consider bare repositories valid.
	 */
	Repository(std::string const &p,
	           RepositoryCreateMode c = RepositoryCreateMode::CreateNormal,
	           bool ab = false);

	~Repository() = default;

	std::string getWorkDirectoryPath() const;
};
}
}

#endif
