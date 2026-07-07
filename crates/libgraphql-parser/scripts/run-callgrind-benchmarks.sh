#!/bin/bash

# Runs the deterministic Callgrind-based benchmarks for the
# libgraphql-parser crate (see benches/callgrind_benchmarks.rs) and
# formats the results as markdown tables.
#
# Unlike the wall-clock Criterion benchmarks (run-benchmarks.sh),
# these run on Valgrind's simulated CPU and measure exact instruction
# counts plus simulated cache behavior. Results are deterministic and
# machine-independent, which makes them suitable for noisy CI runners
# and for tracking regressions over time.
#
# Requirements:
#   - Linux (Valgrind does not support macOS on Apple Silicon)
#   - valgrind on $PATH
#   - iai-callgrind-runner on $PATH, same version as the
#     iai-callgrind crate in Cargo.lock:
#       cargo install iai-callgrind-runner --version <version>
#   - The Shopify Admin schema fixture (fetched via
#     scripts/fetch-shopify-admin-graphql-schema-fixture.sh)
#
# Usage:
#   ./crates/libgraphql-parser/scripts/run-callgrind-benchmarks.sh
#   ./crates/libgraphql-parser/scripts/run-callgrind-benchmarks.sh --format-only
#
# Arguments:
#   --format-only  Skip running the benchmarks and only (re)format the
#                  report from an existing target/iai directory.
#
# The markdown report is written to target/iai/REPORT.md and echoed
# to stdout.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

IAI_DIR="${REPO_ROOT}/target/iai"
REPORT_FILE="${IAI_DIR}/REPORT.md"
SUMMARY_ROOT="${IAI_DIR}/libgraphql-parser/callgrind_benchmarks/callgrind_benches"
SHOPIFY_FIXTURE="${REPO_ROOT}/crates/libgraphql-parser/benches/fixtures"
SHOPIFY_FIXTURE+="/third-party/shopify-admin-schema/schema.graphql"

# ─── Parse Arguments ──────────────────────────────────────

FORMAT_ONLY=false
for arg in "$@"; do
	case "$arg" in
		--format-only) FORMAT_ONLY=true ;;
		*) echo "Unknown argument: $arg" >&2; exit 1 ;;
	esac
done

# ─── Prerequisites ────────────────────────────────────────

assert_installed jq || exit 1
assert_installed cargo || exit 1

if ! $FORMAT_ONLY; then
	if [[ "$OSTYPE" != "linux"* ]]; then
		{
			echo "${UNICODE_RED_X} Valgrind (and therefore this benchmark suite) only runs on"
			echo "  Linux. Use run-benchmarks.sh (Criterion, wall-clock) instead."
		} >&2
		exit 1
	fi

	assert_installed valgrind || exit 1

	IAI_CALLGRIND_VERSION=$(
		cargo metadata --format-version 1 2>/dev/null \
			| jq -r '.packages[] | select(.name == "iai-callgrind") | .version'
	)

	if ! is_installed iai-callgrind-runner; then
		{
			echo "${UNICODE_RED_X} iai-callgrind-runner is not installed (or not on \$PATH)."
			echo "  Install the version matching Cargo.lock with:"
			echo ""
			echo "    cargo install iai-callgrind-runner --version ${IAI_CALLGRIND_VERSION}"
		} >&2
		exit 1
	fi

	if [ ! -f "${SHOPIFY_FIXTURE}" ]; then
		{
			echo "${UNICODE_RED_X} The Shopify Admin schema benchmark fixture is missing. It is"
			echo "  not checked in to the repository and must be fetched first:"
			echo ""
			echo "    ${REPO_ROOT}/crates/libgraphql-parser/scripts/fetch-shopify-admin-graphql-schema-fixture.sh"
		} >&2
		exit 1
	fi
fi

# ─── Run Benchmarks ──────────────────────────────────────

if ! $FORMAT_ONLY; then
	echo ""
	echo "════════════════════════════════════════════════════════"
	echo "  libgraphql-parser Callgrind benchmarks (deterministic)"
	echo "════════════════════════════════════════════════════════"
	echo ""

	cargo bench --package libgraphql-parser --bench callgrind_benchmarks -- \
		--save-summary=json
fi

if [ ! -d "${SUMMARY_ROOT}" ]; then
	{
		echo "${UNICODE_RED_X} No benchmark summaries found under:"
		echo "    ${SUMMARY_ROOT}"
		echo "  Run this script without --format-only first."
	} >&2
	exit 1
fi

# ─── Helpers ──────────────────────────────────────────────

# Read a single integer Callgrind metric (e.g. Ir, EstimatedCycles)
# from an iai-callgrind summary.json. Prints an empty string if the
# summary file is missing.
#
# The metric value is `.Left.Int` on a first-ever run, but when a
# previous run's data exists under target/iai, iai-callgrind diffs
# against it and the shape becomes `.Both[0].Int` (current run
# first). Handle both so re-runs without clearing target/iai still
# format correctly.
read_metric() {
	local group="$1"
	local bench_fn="$2"
	local fixture="$3"
	local metric="$4"
	local json="${SUMMARY_ROOT}/${group}/${bench_fn}.${fixture}/summary.json"
	if [ -f "$json" ]; then
		jq -r \
			--arg metric "$metric" \
			'.profiles[0].summaries.total.summary.Callgrind[$metric].metrics
				| (.Left // .Both[0])
				| .Int // empty' \
			"$json"
	else
		echo ""
	fi
}

# Format an integer with thousands separators (e.g. 12345678 ->
# 12,345,678). Prints N/A for an empty input.
format_int() {
	local val="$1"
	if [ -z "$val" ]; then
		echo "N/A"
		return
	fi
	# Insert separators right-to-left in groups of three. Avoids
	# printf "%'d", which requires locale support that minimal CI
	# containers often lack.
	echo "$val" | rev | sed 's/\([0-9]\{3\}\)/\1,/g' | rev | sed 's/^,//'
}

# Return the 0-based index of the minimum value among the arguments.
# Empty arguments are ignored but still occupy an index, so a missing
# metric for one parser can never shift the winner onto the wrong
# column.
find_min_idx() {
	local min="" idx=-1 i=0 val
	for val in "$@"; do
		if [ -n "$val" ] && { [ -z "$min" ] || [ "$val" -lt "$min" ]; }; then
			min="$val"
			idx=$i
		fi
		i=$((i + 1))
	done
	echo "$idx"
}

# Emit one markdown table comparing all parsers on a single metric
# for every fixture in a benchmark group. The libgraphql lean-mode
# column is informational and excluded from best-value bolding since
# lean mode does strictly less work than the other parsers' default
# configurations (see the fidelity caveat in the report header).
emit_comparison_table() {
	local group="$1"
	local metric="$2"
	local lg_fn="$3"
	local lg_lean_fn="$4"
	local gp_fn="$5"
	local gp_borrowed_fn="$6"
	local ap_fn="$7"
	shift 7
	local fixtures=("$@")

	echo -n "| Input | \`libgraphql-parser\` | \`libgraphql-parser\` (lean)"
	echo -n " | \`graphql-parser\` | \`graphql-parser\` (zero-copy)"
	echo " | \`apollo-parser\` |"
	echo -n "|-------|---------------------|----------------------------"
	echo -n "|------------------|------------------------------"
	echo "|-----------------|"

	local fixture
	for fixture in "${fixtures[@]}"; do
		local lg lg_lean gp gp_borrowed ap
		lg=$(read_metric "$group" "$lg_fn" "$fixture" "$metric")
		lg_lean=$(read_metric "$group" "$lg_lean_fn" "$fixture" "$metric")
		gp=$(read_metric "$group" "$gp_fn" "$fixture" "$metric")
		gp_borrowed=$(read_metric "$group" "$gp_borrowed_fn" "$fixture" "$metric")
		ap=$(read_metric "$group" "$ap_fn" "$fixture" "$metric")

		local min_idx
		min_idx=$(find_min_idx "$lg" "$gp" "$gp_borrowed" "$ap")

		local cells=()
		local vals=("$lg" "$gp" "$gp_borrowed" "$ap")
		local j
		for j in 0 1 2 3; do
			local formatted
			formatted=$(format_int "${vals[$j]}")
			if [ "$j" -eq "$min_idx" ]; then
				cells+=("**${formatted}**")
			else
				cells+=("${formatted}")
			fi
		done

		echo -n "| ${fixture} | ${cells[0]} | $(format_int "$lg_lean")"
		echo " | ${cells[1]} | ${cells[2]} | ${cells[3]} |"
	done
}

# ─── Detect Environment Metadata ─────────────────────────

BENCH_DATE="$(date +%Y-%m-%d)"
RUSTC_VERSION="$(rustc --version)"
VALGRIND_VERSION="$(valgrind --version 2>/dev/null || echo "valgrind (version unknown)")"

GRAPHQL_PARSER_VERSION=$(
	cargo metadata --format-version 1 2>/dev/null \
		| jq -r '.packages[] | select(.name == "graphql-parser") | .version'
)
APOLLO_PARSER_VERSION=$(
	cargo metadata --format-version 1 2>/dev/null \
		| jq -r '.packages[] | select(.name == "apollo-parser") | .version'
)

SCHEMA_FIXTURES=("small" "medium" "large" "starwars" "github" "shopify_admin")
EXEC_FIXTURES=("simple" "complex" "nested_10" "nested_30" "many_ops_50")

# ─── Generate Report ─────────────────────────────────────

mkdir -p "${IAI_DIR}"

{
	echo "# libgraphql-parser Callgrind Benchmark Report"
	echo ""
	echo "> **Measured:** ${BENCH_DATE} under ${VALGRIND_VERSION} (Callgrind,"
	echo "> \`--cache-sim=yes\`), ${RUSTC_VERSION}, \`bench\` profile."
	echo "> Comparison parsers: \`graphql-parser\` ${GRAPHQL_PARSER_VERSION},"
	echo "> \`apollo-parser\` ${APOLLO_PARSER_VERSION}."
	echo ">"
	echo "> All numbers are deterministic simulated-CPU measurements:"
	echo "> **Instructions** is the exact count of CPU instructions executed;"
	echo "> **Estimated Cycles** is derived from Callgrind's cache simulation"
	echo "> (\`L1 hits + 5 * LL hits + 35 * RAM hits\`). Lower is better. These are"
	echo "> a machine-independent proxy for relative performance, not"
	echo "> wall-clock time."
	echo ">"
	echo "> **Fidelity caveat:** the compared parsers do different amounts of"
	echo "> work. \`libgraphql-parser\` (default) and \`apollo-parser\` both retain"
	echo "> lossless syntax/trivia information; \`graphql-parser\` and"
	echo "> \`libgraphql-parser\` (lean) produce a semantic AST only. The"
	echo "> \`graphql-parser\` column uses its owned-\`String\` mode (matching the"
	echo "> Criterion suite) while (zero-copy) uses its borrowed \`&str\` mode."
	echo "> The fairest single-axis comparisons are lean vs. \`graphql-parser\`"
	echo "> and default vs. \`apollo-parser\`."
	echo ""

	echo "## Schema Document Parsing"
	echo ""
	echo "### Instructions"
	echo ""
	emit_comparison_table "schema_parse" "Ir" \
		"schema_libgraphql" "schema_libgraphql_lean" \
		"schema_graphql_parser" "schema_graphql_parser_borrowed" \
		"schema_apollo_parser" \
		"${SCHEMA_FIXTURES[@]}"
	echo ""
	echo "### Estimated Cycles"
	echo ""
	emit_comparison_table "schema_parse" "EstimatedCycles" \
		"schema_libgraphql" "schema_libgraphql_lean" \
		"schema_graphql_parser" "schema_graphql_parser_borrowed" \
		"schema_apollo_parser" \
		"${SCHEMA_FIXTURES[@]}"
	echo ""

	echo "## Executable Document Parsing"
	echo ""
	echo "### Instructions"
	echo ""
	emit_comparison_table "executable_parse" "Ir" \
		"executable_libgraphql" "executable_libgraphql_lean" \
		"executable_graphql_parser" "executable_graphql_parser_borrowed" \
		"executable_apollo_parser" \
		"${EXEC_FIXTURES[@]}"
	echo ""
	echo "### Estimated Cycles"
	echo ""
	emit_comparison_table "executable_parse" "EstimatedCycles" \
		"executable_libgraphql" "executable_libgraphql_lean" \
		"executable_graphql_parser" "executable_graphql_parser_borrowed" \
		"executable_apollo_parser" \
		"${EXEC_FIXTURES[@]}"
	echo ""

	echo "## Lexer (Tokenization Only)"
	echo ""
	echo "| Input | Instructions | Estimated Cycles |"
	echo "|-------|--------------|------------------|"
	for fixture in "${SCHEMA_FIXTURES[@]}"; do
		ir=$(read_metric "lexer" "lexer_libgraphql" "$fixture" "Ir")
		cycles=$(read_metric "lexer" "lexer_libgraphql" "$fixture" "EstimatedCycles")
		echo "| ${fixture} | $(format_int "$ir") | $(format_int "$cycles") |"
	done
	echo ""
} > "${REPORT_FILE}"

cat "${REPORT_FILE}"

echo "" >&2
echo "${UNICODE_GREEN_CHECK} Report written to ${REPORT_FILE}" >&2
