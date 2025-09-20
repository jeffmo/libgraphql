#!/bin/bash

cd "${GITHUB_WORKSPACE}"
cargo clippy --tests -- -Dwarnings
