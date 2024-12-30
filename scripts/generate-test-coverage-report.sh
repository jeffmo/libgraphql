#!/bin/bash

set -e

CARGO_LLVM_COV_INSTALLED="$(cargo --list|grep llvm-cov)"
if [ -z "${CARGO_LLVM_COV_INSTALLED}" ]; then
  >&2 echo "It looks like cargo-llvm-cov is not installed!"
  >&2 echo "Please install it before running this script:"
  >&2 echo
  >&2 echo "    https://github.com/taiki-e/cargo-llvm-cov"
  >&2 echo
  exit 1
fi

echo "Generating coverage report..."
cargo llvm-cov --all-features --workspace --html

# If this is an interactive shell AND macos, prompt to open the report in a
# browser
if [ -t 0 ] && [[ "$OSTYPE" == "darwin"* ]]; then
  echo
  read -r -p "Open the report in a browser? [y/N] " response
  case "$response" in
    [yY])
      CARGO_PKG_ROOT_DIR="$(dirname $(cargo locate-project --message-format plain))"
      open "${CARGO_PKG_ROOT_DIR}"/target/llvm-cov/html/index.html
      ;;

    *)
      echo "Not opening report."
      ;;
  esac
fi
