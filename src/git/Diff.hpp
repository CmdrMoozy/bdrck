#ifndef bdrck_git_Diff_HPP
#define bdrck_git_Diff_HPP

#include <functional>
#include <string>
#include <experimental/optional>

#include <git2.h>
#include <git2/sys/diff.h>

#include "bdrck/git/Wrapper.hpp"

namespace bdrck
{
namespace git
{
class Repository;
class Tree;

class DiffOptions
{
public:
	DiffOptions();

	DiffOptions(DiffOptions const &) = default;
	DiffOptions(DiffOptions &&) = default;
	DiffOptions &operator=(DiffOptions const &) = default;
	DiffOptions &operator=(DiffOptions &&) = default;

	~DiffOptions() = default;

	git_diff_options *get();
	git_diff_options const *get() const;

private:
	git_diff_options options;
};

class Diff : public Wrapper<git_diff, git_diff_free>
{
private:
	typedef Wrapper<git_diff, git_diff_free> base_type;

public:
	typedef std::function<bool(git_diff_delta const &, float)>
	        file_callback;
	typedef std::function<bool(git_diff_delta const &,
	                           git_diff_hunk const &)> hunk_callback;
	typedef std::function<bool(git_diff_delta const &,
	                           git_diff_binary const &)> binary_callback;
	typedef std::function<bool(git_diff_delta const &,
	                           git_diff_hunk const &,
	                           git_diff_line const &)> line_callback;

	/*!
	 * Compute a diff between two arbitrary trees.
	 */
	Diff(Repository &repository, Tree &&oldTree, Tree &&newTree,
	     DiffOptions const &options = DiffOptions());

	/*!
	 * Compute a diff between an arbitrary tree and the working directory.
	 */
	Diff(Repository &repository, Tree &&oldTree, bool withIndex = true,
	     DiffOptions const &options = DiffOptions());

	/*!
	 * Compute a diff between two arbitrary revision specifications.
	 */
	Diff(Repository &repository, std::string const &oldRevspec,
	     std::string const &newRevspec,
	     DiffOptions const &options = DiffOptions());

	/*!
	 * Compute a diff between an arbitrary revision specification and the
	 * working directory.
	 */
	Diff(Repository &repository, std::string const &oldRevspec,
	     bool withIndex = true, DiffOptions const &options = DiffOptions());

	~Diff() = default;

	void foreach(
	        std::experimental::optional<file_callback> const &fileCallback,
	        std::experimental::optional<hunk_callback> const &hunkCallback =
	                std::experimental::nullopt,
	        std::experimental::optional<binary_callback> const
	                &binaryCallback = std::experimental::nullopt,
	        std::experimental::optional<line_callback> const &lineCallback =
	                std::experimental::nullopt);
};

class DiffStats : public Wrapper<git_diff_stats, git_diff_stats_free>
{
private:
	typedef Wrapper<git_diff_stats, git_diff_stats_free> base_type;

public:
	DiffStats(Diff &diff);

	~DiffStats() = default;
};

class DiffPerfData
{
public:
	DiffPerfData(Diff const &diff);

	DiffPerfData(DiffPerfData const &) = default;
	DiffPerfData(DiffPerfData &&) = default;
	DiffPerfData &operator=(DiffPerfData const &) = default;
	DiffPerfData &operator=(DiffPerfData &&) = default;

	~DiffPerfData() = default;

	git_diff_perfdata *get();
	git_diff_perfdata const *get() const;

private:
	git_diff_perfdata data;
};
}
}

#endif
