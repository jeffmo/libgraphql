#!/bin/bash

cd "${GITHUB_WORKSPACE}"
# TODO: Make clippy warnings fail this CI job...
#       (For now we have some "unused code" warnings that should eventually go
#       away with more tests)
#cargo clippy -- -Dwarnings
cargo clippy
