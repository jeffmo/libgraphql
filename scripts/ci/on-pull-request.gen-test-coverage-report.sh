#!/bin/bash

cd "${GITHUB_WORKSPACE}"
cargo llvm-cov --all-features --workspace --html
