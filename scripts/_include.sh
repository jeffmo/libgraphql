#!/bin/bash

UNICODE_GREEN_CHECK="✅"
UNICODE_RED_X="❌"

# Assert that a command is installed, with helpful error messages
#
# Checks if a command is installed. If not, outputs an error message to stderr
# with platform-specific installation instructions using the appropriate package manager.
#
# Works cross-platform on Linux, macOS, and Windows (with bash environments)
#
# Usage: assert_installed "command_name"
# Returns: 0 if command is found, 1 if not found
assert_installed() {
	local cmd="$1"

	if is_installed "$cmd"; then
		return 0
	fi

	# Detect platform and set appropriate package manager
	local platform
	local pkg_manager
	local install_cmd
	local pkg_name

	if [[ "$OSTYPE" == "darwin"* ]]; then
		# macOS
		platform="macOS"
		pkg_manager="Homebrew"

		if is_installed "brew"; then
			# Use 'brew which-formula' to find the package that installs the command
			pkg_name=$(brew which-formula "$cmd" 2>/dev/null)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="brew install $pkg_name"
			fi
		else
			install_cmd=""
		fi
	elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
		# Windows with bash (Git Bash, MSYS2, Cygwin)
		platform="Windows"
		pkg_manager="winget"

		if is_installed "winget"; then
			# Use 'winget search --command' to find the package that installs the command
			pkg_name=$(winget search --command "$cmd" 2>/dev/null | head -1)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="winget install $pkg_name"
			fi
		else
			install_cmd=""
		fi
	else
		# Linux and other Unix-like systems
		platform="Linux"

		# Detect Linux distribution and set appropriate package manager
		if is_installed "apt-cache"; then
			pkg_manager="apt"
			# Use 'apt-cache search' to find the package
			pkg_name=$(apt-cache search "^$cmd\$" 2>/dev/null | awk '{print $1}' | head -1)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="apt install $pkg_name"
			fi
		elif is_installed "dnf"; then
			pkg_manager="dnf"
			# Use 'dnf search' to find the package
			pkg_name=$(dnf search "$cmd" 2>/dev/null | grep -i "Name" | awk '{print $3}' | head -1)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="dnf install $pkg_name"
			fi
		elif is_installed "pacman"; then
			pkg_manager="pacman"
			# Use 'pacman -Ss' to search for the package
			pkg_name=$(pacman -Ss "^$cmd\$" 2>/dev/null | grep "^[^ ]" | awk '{print $1}' | cut -d'/' -f2 | head -1)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="pacman -S $pkg_name"
			fi
		elif is_installed "zypper"; then
			pkg_manager="zypper"
			# Use 'zypper search' to find the package
			pkg_name=$(zypper search -t package "$cmd" 2>/dev/null | grep "^i\|^  " | awk '{print $3}' | head -1)

			if [[ -n "$pkg_name" ]]; then
				install_cmd="zypper install $pkg_name"
			fi
		else
			pkg_manager=""
		fi
	fi

	# Output error message to stderr
	{
    printf "%s" "${UNICODE_RED_X} The \`$cmd\` command is required for this script"
		if [[ -n "$install_cmd" ]]; then
			echo ". To install, run:"
      echo
      echo "    $install_cmd"
      echo
		else
      echo ", but does not appear to be installed."
		fi
	} >&2

	return 1
}

# Assert that a cargo-installed binary is installed, with helpful error messages
#
# Checks if a binary installed via 'cargo install' is present.
# If not, outputs an error message to stderr with instructions to install it.
#
# Works cross-platform on Linux, macOS, and Windows (with bash environments)
#
# Usage: assert_cargo_installed "binary_name"
# Returns: 0 if the cargo-installed binary is found, 1 if not found
assert_cargo_installed() {
	local binary="$1"

	if is_cargo_installed "$binary"; then
		return 0
	fi

	# Output error message to stderr
	{
		echo "${UNICODE_RED_X} The \`$binary\` cargo binary is required for this script. To install, run:"
		echo
		echo "    cargo install $binary"
		echo
	} >&2

	return 1
}

# Get the package version from a Cargo.toml file
#
# Extracts the version number from a Cargo.toml file using the cargo CLI.
# Works reliably across different Cargo.toml formats.
#
# Usage: get_crate_local_version <path_to_cargo_toml>
# Returns: 0 on success, 1 if the version cannot be determined
# Outputs: The version number (e.g., "0.1.0")
get_crate_local_version() {
	assert_installed jq || return 1

	local cargo_toml="$1"

	if [[ ! -f "$cargo_toml" ]]; then
		echo "Error: Cargo.toml file not found at $cargo_toml" >&2
		return 1
	fi

  local pkgid=$(
    cargo pkgid \
      --manifest-path "$(to_absolute_path "${cargo_toml}")" \
      2>/dev/null
  )
  local crate_name=$(basename $(echo "${pkgid}" | cut -d: -f2 | cut -d# -f1))
  local crate_version=$(echo "${pkgid}" | cut -d# -f2)
  echo "${crate_version}"
}

# Get the latest published version of a crate from crates.io
#
# Fetches the latest published version of a crate by name from crates.io
# using the cargo info command.
#
# Usage: get_crate_published_version <crate_name>
# Returns: 0 on success, 1 if the version cannot be determined
# Outputs: The version number (e.g., "0.1.0")
get_crate_published_version() {
	local crate_name="$1"

	if [[ -z "$crate_name" ]]; then
		echo "Error: crate_name is required" >&2
		return 1
	fi

	# Use cargo info to fetch the latest published version from crates.io
	# Parse the output to extract the version number
	cargo info "$crate_name" --registry crates-io 2>/dev/null \
		| grep "^version" | head -1 | awk '{print $2}'
}

# Get the absolute path to the repository root
#
# Caches the result on first call and returns the cached value on subsequent calls
#
# Usage: get_repo_root
# Returns: 0 on success, 1 if the repository root cannot be determined
# Outputs: The absolute path to the repository root
_CACHED_REPO_ROOT=""
get_repo_root() {
	# Return cached value if available
	if [[ -n "$_CACHED_REPO_ROOT" ]]; then
		echo "$_CACHED_REPO_ROOT"
		return 0
	fi

	# Calculate repository root as parent of scripts directory
	_CACHED_REPO_ROOT="$(to_absolute_path ".." "$(dirname "${BASH_SOURCE[0]}")")" || return 1
	echo "$_CACHED_REPO_ROOT"
	return 0
}

# Check if a cargo-installed binary is installed
#
# Checks if a binary installed via 'cargo install' is present
# by examining the output of 'cargo install --list'
#
# Usage: is_cargo_installed "binary_name"
# Returns: 0 if the cargo-installed binary is found, 1 if not found
is_cargo_installed() {
	local binary="$1"

	cargo install --list 2>/dev/null | grep -q "^${binary} "
}

# Check if a command is installed and available in PATH
#
# Works cross-platform on Linux, macOS, and Windows (with bash environments)
#
# Usage: is_installed "command_name"
# Returns: 0 if command is found, 1 if not found
is_installed() {
	command -v "$1" > /dev/null 2>&1
}

# Convert a path to an absolute path
#
# Works cross-platform on Linux, macOS, and Windows (with bash environments)
# Handles both files and directories (existing or non-existing)
#
# Usage: to_absolute_path <path> [base_dir]
#   path: The path to convert (may be relative or absolute, file or directory)
#   base_dir: Optional base directory for resolving relative paths (defaults to current directory)
#
# Returns: 0 on success, 1 if the path cannot be resolved
# Outputs: The absolute path
to_absolute_path() {
	local path="$1"
	local base_dir="${2:-.}"

	# Expand tilde in home directory references
	path="${path/#\~/$HOME}"
	base_dir="${base_dir/#\~/$HOME}"

	# If path is already absolute, return it as-is
	if [[ "$path" = /* ]] || [[ "$path" =~ ^[a-zA-Z]: ]]; then
		echo "$path"
		return 0
	fi

	# For relative paths, resolve them relative to base_dir
	(
		cd "$base_dir" 2>/dev/null || return 1

		# Handle the path based on whether it exists and what type it is
		if [[ -d "$path" ]]; then
			# Path is an existing directory - cd into it
			cd "$path" && pwd
		elif [[ -e "$path" ]]; then
			# Path is an existing file - resolve its directory and append filename
			echo "$(cd "$(dirname "$path")" && pwd)/$(basename "$path")"
		else
			# Path doesn't exist - resolve parent directory and append path
			echo "$(cd "$(dirname "$path")" 2>/dev/null && pwd)/$(basename "$path")"
		fi
	) || return 1
}

# Trim leading and trailing whitespace from a string
#
# Removes all leading and trailing whitespace characters (spaces, tabs, newlines, etc)
# from the input string, including newlines and all whitespace character classes.
#
# Usage: trim_str "  string with spaces  "
# Outputs: The trimmed string
trim_str() {
	local str="$1"
	# Remove leading whitespace using extended regex
	str=$(echo "$str" | sed 's/^[[:space:]]*//g')
	# Remove trailing whitespace using extended regex
	str=$(echo "$str" | sed 's/[[:space:]]*$//g')
	echo "$str"
}
