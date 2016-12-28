#ifndef bdrck_git_Library_HPP
#define bdrck_git_Library_HPP

#include <memory>
#include <mutex>

namespace bdrck
{
namespace git
{
/*!
 * \brief A RAII-style class to initialize the Libary singleton.
 */
class LibraryInstance
{
public:
	LibraryInstance();

	LibraryInstance(LibraryInstance const &) = delete;
	LibraryInstance(LibraryInstance &&) = default;
	LibraryInstance &operator=(LibraryInstance const &) = delete;
	LibraryInstance &operator=(LibraryInstance &&) = default;

	~LibraryInstance();
};

/*!
 * \brief A singleton which handles initializing libgit2.
 */
class Library
{
public:
	static bool isInitialized();

	Library(Library const &) = delete;
	Library(Library &&) = default;
	Library &operator=(Library const &) = delete;
	Library &operator=(Library &&) = default;

	~Library();

private:
	friend class LibraryInstance;

	static std::mutex mutex;
	static std::unique_ptr<Library> instance;

	Library();
};
}
}

#endif
