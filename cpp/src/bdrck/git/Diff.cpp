#include "Diff.hpp"

#include <exception>
#include <utility>

#include <boost/optional/optional.hpp>

#include "bdrck/git/Object.hpp"
#include "bdrck/git/Repository.hpp"
#include "bdrck/git/Tree.hpp"
#include "bdrck/git/checkReturn.hpp"

namespace
{
struct DiffForeachContext
{
	bdrck::git::Diff::file_callback fileCallback;
	bdrck::git::Diff::hunk_callback hunkCallback;
	bdrck::git::Diff::binary_callback binaryCallback;
	bdrck::git::Diff::line_callback lineCallback;
	boost::optional<std::exception_ptr> error;

	DiffForeachContext(bdrck::git::Diff::file_callback const &fc,
	                   bdrck::git::Diff::hunk_callback const &hc,
	                   bdrck::git::Diff::binary_callback const &bc,
	                   bdrck::git::Diff::line_callback const &lc)
	        : fileCallback(fc),
	          hunkCallback(hc),
	          binaryCallback(bc),
	          lineCallback(lc),
	          error(boost::none)
	{
	}
};

template <typename... Arg>
int callbackImpl(std::function<bool(Arg...)> const &callback,
                 boost::optional<std::exception_ptr> &error, Arg... arg)
{
	if(!callback)
		return 0;

	try
	{
		return callback(std::forward<Arg>(arg)...) ? 0 : -1;
	}
	catch(...)
	{
		error.emplace(std::current_exception());
		return -1;
	}
}

int fileCallbackImpl(git_diff_delta const *delta, float progress, void *payload)
{
	auto context = static_cast<DiffForeachContext *>(payload);
	return callbackImpl<git_diff_delta const &, float>(
	        context->fileCallback, context->error, *delta, progress);
}

int hunkCallbackImpl(git_diff_delta const *delta, git_diff_hunk const *hunk,
                     void *payload)
{
	auto context = static_cast<DiffForeachContext *>(payload);
	return callbackImpl<git_diff_delta const &, git_diff_hunk const &>(
	        context->hunkCallback, context->error, *delta, *hunk);
}

int binaryCallbackImpl(git_diff_delta const *delta,
                       git_diff_binary const *binary, void *payload)
{
	auto context = static_cast<DiffForeachContext *>(payload);
	return callbackImpl<git_diff_delta const &, git_diff_binary const &>(
	        context->binaryCallback, context->error, *delta, *binary);
}

int lineCallbackImpl(git_diff_delta const *delta, git_diff_hunk const *hunk,
                     git_diff_line const *line, void *payload)
{
	auto context = static_cast<DiffForeachContext *>(payload);
	return callbackImpl<git_diff_delta const &, git_diff_hunk const &,
	                    git_diff_line const &>(
	        context->lineCallback, context->error, *delta, *hunk, *line);
}
}

namespace bdrck
{
namespace git
{
DiffOptions::DiffOptions() : options()
{
	checkReturn(git_diff_init_options(&options, GIT_DIFF_OPTIONS_VERSION));
}

git_diff_options *DiffOptions::get()
{
	return &options;
}

git_diff_options const *DiffOptions::get() const
{
	return &options;
}

Diff::Diff(Repository &repository, Tree &&oldTree, Tree &&newTree,
           DiffOptions const &options)
        : base_type(git_diff_tree_to_tree, repository.get(), oldTree.get(),
                    newTree.get(), options.get())
{
}

Diff::Diff(Repository &repository, Tree &&oldTree, bool withIndex,
           DiffOptions const &options)
        : base_type(withIndex ? git_diff_tree_to_workdir_with_index
                              : git_diff_tree_to_workdir,
                    repository.get(), oldTree.get(), options.get())
{
}

Diff::Diff(Repository &repository, std::string const &oldRevspec,
           std::string const &newRevspec, DiffOptions const &options)
        : Diff(repository, Tree(Object(oldRevspec, repository)),
               Tree(Object(newRevspec, repository)), options)
{
}

Diff::Diff(Repository &repository, std::string const &oldRevspec,
           bool withIndex, DiffOptions const &options)
        : Diff(repository, Tree(Object(oldRevspec, repository)), withIndex,
               options)
{
}

void Diff::foreach(file_callback const &fileCallback,
                   hunk_callback const &hunkCallback,
                   binary_callback const &binaryCallback,
                   line_callback const &lineCallback)
{
	DiffForeachContext context(fileCallback, hunkCallback, binaryCallback,
	                           lineCallback);
	git_diff_foreach(get(), fileCallbackImpl, binaryCallbackImpl,
	                 hunkCallbackImpl, lineCallbackImpl, &context);
	if(!!context.error)
		std::rethrow_exception(context.error.value());
}

DiffStats::DiffStats(Diff &diff) : base_type(git_diff_get_stats, diff.get())
{
}

DiffPerfData::DiffPerfData(Diff const &diff) : data()
{
	checkReturn(git_diff_get_perfdata(&data, diff.get()));
}

git_diff_perfdata *DiffPerfData::get()
{
	return &data;
}

git_diff_perfdata const *DiffPerfData::get() const
{
	return &data;
}
}
}