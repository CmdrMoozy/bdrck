#ifndef bdrck_params_Option_HPP
#define bdrck_params_Option_HPP

#include <cstddef>
#include <initializer_list>
#include <iterator>
#include <memory>
#include <string>
#include <vector>
#include <experimental/optional>

namespace bdrck
{
namespace params
{
class OptionSet;

/*!
 * An option is a non-positional parameter to a command. Options can either be
 * normal options or flags. Normal options must be passed by name along with a
 * value. Flags are options whose value is either true or false, and is false
 * by default. Passing a flag by name means flipping its value to true.
 */
struct Option
{
public:
	/*!
	 * Helper for constructing a required option. This option may have a
	 * default value. But, importantly, it will always have some value
	 * inside the command function.
	 *
	 * See Option's private constructor's documentation for more info.
	 *
	 * \param n The full name for this option.
	 * \param h The help message for this option.
	 * \param sn The short name for this option, if any.
	 * \param dv The default value for this option, if any.
	 * \return The newly-constructed option.
	 */
	static Option required(std::string const &n, std::string const &h,
	                       std::experimental::optional<char> const &sn =
	                               std::experimental::nullopt,
	                       std::experimental::optional<std::string> const
	                               &dv = std::experimental::nullopt);

	/*!
	 * Overloaded helper for constructing a required option. Allows the
	 * default value to be a string literal.
	 *
	 * \param n The full name for this option.
	 * \param h The help message for this option.
	 * \param sn The short name for this option, if any.
	 * \param dv The default value for this option.
	 * \return The newly-constructed option.
	 */
	static Option required(std::string const &n, std::string const &h,
	                       std::experimental::optional<char> const &sn,
	                       std::string const &dv);

	/*!
	 * Helper for constructing an optional option. This option does not
	 * have a default value, and it may have no value to access inside the
	 * command function.
	 *
	 * See Option's private constructor's documentation for more info.
	 *
	 * \param n The full name for this option.
	 * \param h The help message for this option.
	 * \param sn The short name for this option, if any.
	 * \return The newly-constructed option.
	 */
	static Option optional(std::string const &n, std::string const &h,
	                       std::experimental::optional<char> const &sn =
	                               std::experimental::nullopt);

	/*!
	 * Helper for constructing a flag option. This option's value is either
	 * true or false, and is false unless it is explicitly passed to the
	 * command as an argument.
	 *
	 * See Option's private constructor's documentation for more info.
	 *
	 * \param n The full name for this option.
	 * \param h The help message for this option.
	 * \param sn The short name for this option, if any.
	 * \return The newly-constructed option.
	 */
	static Option flag(std::string const &n, std::string const &h,
	                   std::experimental::optional<char> const &sn =
	                           std::experimental::nullopt);

	std::string name;
	std::string help;
	std::experimental::optional<char> shortName;
	std::experimental::optional<std::string> defaultValue;
	bool isOptional;
	bool isFlag;

private:
	friend class OptionSet;

	/*!
	 * Constructs a new option. Options must have at least a long name and
	 * some help text explaining what the option does. Options can also
	 * have a single-character short name, which can be used instead of the
	 * full name.
	 *
	 * Options can have a default value, which is the value passed to the
	 * command function if the option is not specified explicitly by the
	 * user.
	 *
	 * Alternatively, options can be optional, in which case if the user
	 * does not explicitly pass the option to the command, then no value
	 * will be available for the option in the command function.
	 *
	 * Finally, options can be flags instead of value-based options. In
	 * this case, the only values the option can have are true or false.
	 * If the flag is not added to the command-line arguments, then its
	 * value is false by default. Otherwise, its value is true.
	 *
	 * \param n The full name of the option, to be passed with --.
	 * \param h The help message for this option.
	 * \param sn The short name, to be passed with -, if any.
	 * \param dv The default value for this option, if any.
	 * \param o Whether or not this option is optional (can have no value).
	 * \param f Whether or not this option is a flag.
	 */
	Option(std::string const &n, std::string const &h,
	       std::experimental::optional<char> const &sn,
	       std::experimental::optional<std::string> const &dv, bool o,
	       bool f);

	/*!
	 * Construct a new Option with the given name, and default values for
	 * the rest of the possible parameters. This constructor is mainly
	 * useful when searching for Options in an OptionSet.
	 *
	 * \param n The full name of the option.
	 */
	Option(std::string const &n);
};

class OptionSetConstIterator
{
public:
	typedef std::size_t difference_type;
	typedef Option value_type;
	typedef Option const &reference;
	typedef Option const *pointer;
	typedef std::forward_iterator_tag iterator_category;

	OptionSetConstIterator();

	OptionSetConstIterator(OptionSetConstIterator const &) = default;
	OptionSetConstIterator &
	operator=(OptionSetConstIterator const &) = default;

	~OptionSetConstIterator() = default;

	Option const &operator*() const;
	OptionSetConstIterator &operator++();

	bool operator==(OptionSetConstIterator const &o);
	bool operator!=(OptionSetConstIterator const &o);

private:
	friend class OptionSet;

	OptionSetConstIterator(std::vector<std::shared_ptr<Option>> const &v);

	std::shared_ptr<Option> const *data;
	std::size_t length;
	std::size_t current;
};

class OptionSet
{
private:
	struct OptionSetImpl;

public:
	OptionSet(std::initializer_list<Option> const &o);

	OptionSet(OptionSet const &o);
	OptionSet(OptionSet &&o);
	OptionSet &operator=(OptionSet const &o);
	OptionSet &operator=(OptionSet &&o);

	~OptionSet();

	std::size_t size() const;

	OptionSetConstIterator begin() const;
	OptionSetConstIterator end() const;

	Option const *find(std::string const &parameter) const;

private:
	std::unique_ptr<OptionSetImpl> impl;
};
}
}

#endif
