#!/bin/bash

# Updates the comparison ("competitor") GraphQL parsers used by the
# libgraphql-parser benchmarks to their latest stable releases on
# crates.io, so that periodic CI benchmark runs always compare
# against what users would get from a fresh `cargo add` today.
#
# This rewrites the version requirements in the workspace Cargo.toml
# to exact-pinned latest versions (e.g. `apollo-parser = "=0.9.2"`)
# and runs `cargo update` for each crate. The changes are meant for
# transient CI use only and are NOT intended to be committed.
#
# Note: a new competitor release with breaking API changes can cause
# the benchmark build to fail. That failure is a useful signal (the
# benchmark code needs updating), not something to paper over.
#
# Usage:
#   ./crates/libgraphql-parser/scripts/ci/parser-benchmarks.use-latest-competitor-parsers.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

WORKSPACE_CARGO_TOML="${REPO_ROOT}/Cargo.toml"
COMPETITOR_CRATES=("graphql-parser" "apollo-parser")

# ─── Prerequisites ────────────────────────────────────────

assert_installed cargo || exit 1
assert_installed curl || exit 1
assert_installed jq || exit 1

# ─── Helpers ──────────────────────────────────────────────

# Print the latest non-yanked, non-prerelease version of a crate by
# querying the crates.io sparse index (https://index.crates.io).
latest_stable_crate_version() {
	local crate_name="$1"
	local shard="${crate_name:0:2}/${crate_name:2:2}"
	curl --fail --silent --show-error "https://index.crates.io/${shard}/${crate_name}" \
		| jq -r '
			select(.yanked | not)
			| .vers
			| select(test("-") | not)
		' \
		| tail -1
}

# ─── Update Each Competitor Crate ────────────────────────

for crate in "${COMPETITOR_CRATES[@]}"; do
	latest="$(latest_stable_crate_version "$crate")"
	if [ -z "$latest" ]; then
		echo "✘ Could not determine the latest version of ${crate}" >&2
		exit 1
	fi

	echo "Pinning ${crate} to latest stable release: ${latest}"

	if ! grep -qE "^${crate} = \"[^\"]*\"$" "${WORKSPACE_CARGO_TOML}"; then
		{
			echo "✘ Could not find a '${crate} = \"...\"' entry in:"
			echo "    ${WORKSPACE_CARGO_TOML}"
		} >&2
		exit 1
	fi

	sed -i -E \
		"s|^${crate} = \"[^\"]*\"$|${crate} = \"=${latest}\"|" \
		"${WORKSPACE_CARGO_TOML}"

	cargo update --package "$crate" --precise "$latest"
done

echo ""
echo "✔ Competitor parser versions now in the dependency graph:"
cargo metadata --format-version 1 \
	| jq -r '
		.packages[]
		| select(.name == "graphql-parser" or .name == "apollo-parser")
		| "  \(.name) \(.version)"
	'
