#!/bin/bash

# Runs benchmarks for the libgraphql-parser crate with aggressive
# Criterion settings to obtain high-confidence results, then formats
# the output as markdown tables suitable for the README.
#
# Usage:
#   ./crates/libgraphql-parser/scripts/run-benchmarks.sh
#   ./crates/libgraphql-parser/scripts/run-benchmarks.sh --quick
#
# Arguments:
#   --quick  Use faster (less precise) settings for a quick sanity check
#
# Environment variable overrides:
#   BENCH_MEASUREMENT_TIME  Seconds of measurement per benchmark (default: 20)
#   BENCH_SAMPLE_SIZE       Number of samples per benchmark (default: 300)
#   BENCH_WARM_UP_TIME      Seconds of warm-up per benchmark (default: 5)
#   BENCH_CONFIDENCE_LEVEL  Confidence level for CI (default: 0.99)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

CRITERION_DIR="${REPO_ROOT}/target/criterion"

# ─── Settings ─────────────────────────────────────────────

if [[ "${1:-}" == "--quick" ]]; then
	MEASUREMENT_TIME="${BENCH_MEASUREMENT_TIME:-5}"
	SAMPLE_SIZE="${BENCH_SAMPLE_SIZE:-100}"
	WARM_UP_TIME="${BENCH_WARM_UP_TIME:-3}"
	CONFIDENCE_LEVEL="${BENCH_CONFIDENCE_LEVEL:-0.95}"
else
	MEASUREMENT_TIME="${BENCH_MEASUREMENT_TIME:-20}"
	SAMPLE_SIZE="${BENCH_SAMPLE_SIZE:-300}"
	WARM_UP_TIME="${BENCH_WARM_UP_TIME:-5}"
	CONFIDENCE_LEVEL="${BENCH_CONFIDENCE_LEVEL:-0.99}"
fi

# ─── Prerequisites ────────────────────────────────────────

assert_installed jq || exit 1
assert_installed cargo || exit 1

# ─── Helpers ──────────────────────────────────────────────

# Read the mean point estimate (nanoseconds) from a Criterion
# estimates.json file. Prints empty string if the file is missing.
read_estimate() {
	local json_path="$1"
	if [ -f "$json_path" ]; then
		jq -r '.mean.point_estimate' "$json_path"
	else
		echo ""
	fi
}

# Read the CI width as a percentage of the point estimate.
read_ci_pct() {
	local json_path="$1"
	if [ ! -f "$json_path" ]; then
		echo "N/A"
		return
	fi
	jq -r '
		.mean.confidence_interval as $ci |
		.mean.point_estimate as $pe |
		(($ci.upper_bound - $ci.lower_bound) / $pe * 100) |
		. * 100 | round / 100
	' "$json_path"
}

# Format nanoseconds to a human-readable string with 3 significant
# figures, choosing the most natural unit (ns / µs / ms / s).
format_time() {
	local ns="$1"
	if [ -z "$ns" ] || [ "$ns" = "null" ]; then
		echo "N/A"
		return
	fi
	awk -v ns="$ns" 'BEGIN {
		if (ns < 1000) {
			if      (ns >= 100) printf "%.0f ns", ns
			else if (ns >= 10)  printf "%.1f ns", ns
			else                printf "%.2f ns", ns
		} else if (ns < 1000000) {
			us = ns / 1000
			if      (us >= 100) printf "%.0f µs", us
			else if (us >= 10)  printf "%.1f µs", us
			else                printf "%.2f µs", us
		} else if (ns < 1000000000) {
			ms = ns / 1000000
			if      (ms >= 100) printf "%.0f ms", ms
			else if (ms >= 10)  printf "%.1f ms", ms
			else                printf "%.2f ms", ms
		} else {
			s = ns / 1000000000
			printf "%.2f s", s
		}
	}'
}

# Format throughput in MiB/s given time in ns and size in bytes.
format_throughput() {
	local ns="$1"
	local bytes="$2"
	if [ -z "$ns" ] || [ "$ns" = "null" ]; then
		echo "N/A"
		return
	fi
	awk -v ns="$ns" -v bytes="$bytes" 'BEGIN {
		secs = ns / 1000000000
		mibs = (bytes / secs) / (1024 * 1024)
		printf "~%.0f MiB/s", mibs
	}'
}

# Return the 0-based index of the minimum value among the arguments.
find_min_idx() {
	echo "$@" | awk '{
		min = $1; idx = 0
		for (i = 2; i <= NF; i++) {
			if ($i != "" && ($i + 0) < (min + 0)) { min = $i; idx = i - 1 }
		}
		print idx
	}'
}

# Resolve the Criterion output directory for a lexer benchmark.
# Handles potential name variations (e.g. with/without " (synthetic)").
# Prefers the "(synthetic)" suffix since the current benchmark code uses
# those names for the synthetic fixtures.
find_lexer_dir() {
	local name="$1"
	# Try with " (synthetic)" suffix first (current benchmark names)
	if [ -d "${CRITERION_DIR}/lexer/${name} (synthetic)" ]; then
		echo "${CRITERION_DIR}/lexer/${name} (synthetic)"
		return
	fi
	# Try exact match
	if [ -d "${CRITERION_DIR}/lexer/${name}" ]; then
		echo "${CRITERION_DIR}/lexer/${name}"
		return
	fi
	echo ""
}

# Read the throughput bytes from a Criterion benchmark.json file.
read_throughput_bytes() {
	local dir="$1"
	local json="${dir}/new/benchmark.json"
	if [ -f "$json" ]; then
		jq -r '.throughput.Bytes // empty' "$json"
	else
		echo ""
	fi
}

# ─── Run Benchmarks ──────────────────────────────────────

TOTAL_BENCHMARKS=36
EST_SECONDS=$((TOTAL_BENCHMARKS * (WARM_UP_TIME + MEASUREMENT_TIME)))
EST_MINUTES=$(awk \
	-v s="$EST_SECONDS" \
	'BEGIN { printf "%.0f", s / 60 }' \
)

echo ""
echo "════════════════════════════════════════════════════════"
echo "  libgraphql-parser benchmarks (high-confidence run)"
echo "════════════════════════════════════════════════════════"
echo ""
echo "  Measurement time:  ${MEASUREMENT_TIME}s per benchmark"
echo "  Sample size:       ${SAMPLE_SIZE}"
echo "  Warm-up time:      ${WARM_UP_TIME}s"
echo "  Confidence level:  ${CONFIDENCE_LEVEL}"
echo "  Total benchmarks:  ~${TOTAL_BENCHMARKS}"
echo "  Estimated runtime: ~${EST_MINUTES} minutes"
echo ""

cargo bench --package libgraphql-parser --bench parse_benchmarks -- \
	--measurement-time "$MEASUREMENT_TIME" \
	--sample-size "$SAMPLE_SIZE" \
	--warm-up-time "$WARM_UP_TIME" \
	--confidence-level "$CONFIDENCE_LEVEL"

# ─── Parse & Format Results ──────────────────────────────

SCHEMA_INPUTS=("small" "medium" "large" "starwars" "github")
SCHEMA_LABELS=(
	"small (~1.5 KB)"
	"medium (~106 KB)"
	"large (~500 KB)"
	"starwars (~4 KB)"
	"github (~1.2 MB)"
)

EXEC_INPUTS=("simple" "complex")
EXEC_LABELS=("simple query" "complex query")

PARSERS=("libgraphql_parser" "graphql_parser" "apollo_parser")

LEXER_NAMES=(
	"small_schema"
	"medium_schema"
	"large_schema"
	"starwars_schema"
	"github_schema"
)
LEXER_LABELS=(
	"small (~1.5 KB)"
	"medium (~106 KB)"
	"large (~500 KB)"
	"starwars (~4 KB)"
	"github (~1.2 MB)"
)

# ─── Detect Environment Metadata ─────────────────────────

BENCH_DATE="$(date +%Y-%m-%d)"
RUSTC_VERSION="$(rustc --version)"

# Extract comparison parser versions from cargo metadata
GRAPHQL_PARSER_VERSION=$(
	cargo metadata --format-version 1 2>/dev/null \
		| jq -r '.packages[] | select(.name == "graphql-parser") | .version'
)
APOLLO_PARSER_VERSION=$(
	cargo metadata --format-version 1 2>/dev/null \
		| jq -r '.packages[] | select(.name == "apollo-parser") | .version'
)

# Detect machine info (macOS-specific; graceful fallback)
if [[ "$OSTYPE" == "darwin"* ]]; then
	CHIP="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")"
	ARCH="$(uname -m)"
	RAM_BYTES="$(sysctl -n hw.memsize 2>/dev/null || echo "0")"
	RAM_GB=$((RAM_BYTES / 1073741824))
	MACHINE_INFO="${CHIP} (${ARCH}), ${RAM_GB} GB RAM, macOS"
else
	MACHINE_INFO="$(uname -m), $(uname -s)"
fi

echo ""
echo ""
echo "════════════════════════════════════════════════════════"
echo "  Formatted Results for README"
echo "════════════════════════════════════════════════════════"
echo ""
echo "> **Measured:** ${BENCH_DATE} on ${MACHINE_INFO},"
echo "> ${RUSTC_VERSION}, \`--release\` profile."
echo "> Comparison parsers: \`graphql-parser\` ${GRAPHQL_PARSER_VERSION},"
echo "> \`apollo-parser\` ${APOLLO_PARSER_VERSION}."

# ─── Schema Parsing Table ─────────────────────

echo ""
echo "### Schema Parsing"
echo ""
echo "| Input               | \`libgraphql-parser\` | \`graphql-parser\` | \`apollo-parser\` |"
echo "|---------------------|---------------------|------------------|-----------------|"

for i in "${!SCHEMA_INPUTS[@]}"; do
	input="${SCHEMA_INPUTS[$i]}"
	label="${SCHEMA_LABELS[$i]}"

	vals=()
	for parser in "${PARSERS[@]}"; do
		json="${CRITERION_DIR}/compare_schema_parse/${parser}/${input}/new/estimates.json"
		vals+=("$(read_estimate "$json")")
	done

	min_idx=$(find_min_idx "${vals[@]}")

	row="| $(printf '%-19s' "$label")"
	for j in "${!PARSERS[@]}"; do
		formatted=$(format_time "${vals[$j]}")
		if [ "$j" -eq "$min_idx" ]; then
			cell="**${formatted}**"
		else
			cell="${formatted}"
		fi
		# Pad columns for alignment
		case $j in
			0) row+=" | $(printf '%-19s' "$cell")" ;;
			1) row+=" | $(printf '%-16s' "$cell")" ;;
			2) row+=" | $(printf '%-15s' "$cell")" ;;
		esac
	done
	row+=" |"
	echo "$row"
done

# ─── Executable Document Parsing Table ────────

echo ""
echo "### Executable Document Parsing"
echo ""
echo "| Input             | \`libgraphql-parser\` | \`graphql-parser\` | \`apollo-parser\` |"
echo "|-------------------|---------------------|------------------|-----------------|"

for i in "${!EXEC_INPUTS[@]}"; do
	input="${EXEC_INPUTS[$i]}"
	label="${EXEC_LABELS[$i]}"

	vals=()
	for parser in "${PARSERS[@]}"; do
		json="${CRITERION_DIR}/compare_executable_parse/${parser}/${input}/new/estimates.json"
		vals+=("$(read_estimate "$json")")
	done

	min_idx=$(find_min_idx "${vals[@]}")

	row="| $(printf '%-17s' "$label")"
	for j in "${!PARSERS[@]}"; do
		formatted=$(format_time "${vals[$j]}")
		if [ "$j" -eq "$min_idx" ]; then
			cell="**${formatted}**"
		else
			cell="${formatted}"
		fi
		case $j in
			0) row+=" | $(printf '%-19s' "$cell")" ;;
			1) row+=" | $(printf '%-16s' "$cell")" ;;
			2) row+=" | $(printf '%-15s' "$cell")" ;;
		esac
	done
	row+=" |"
	echo "$row"
done

# ─── Lexer Throughput Table ───────────────────

echo ""
echo "### Lexer Throughput"
echo ""
echo "| Input               | Time     | Throughput  |"
echo "|---------------------|----------|-------------|"

for i in "${!LEXER_NAMES[@]}"; do
	name="${LEXER_NAMES[$i]}"
	label="${LEXER_LABELS[$i]}"
	dir=$(find_lexer_dir "$name")

	if [ -z "$dir" ]; then
		echo "| $(printf '%-19s' "$label") | N/A      | N/A         |"
		continue
	fi

	json="${dir}/new/estimates.json"
	ns=$(read_estimate "$json")
	bytes=$(read_throughput_bytes "$dir")
	time_str=$(format_time "$ns")
	tp_str=$(format_throughput "$ns" "$bytes")

	echo "| $(printf '%-19s' "$label") | $(printf '%-8s' "$time_str") | $(printf '%-11s' "$tp_str") |"
done

# ─── Confidence Interval Report ──────────────

echo ""
echo ""
echo "════════════════════════════════════════════════════════"
echo "  Confidence Interval Widths (${CONFIDENCE_LEVEL} level)"
echo "════════════════════════════════════════════════════════"
echo ""
echo "Narrower is better. Values under 2% indicate highly"
echo "reproducible measurements."
echo ""

printf "%-45s  %s\n" "Benchmark" "CI width"
printf "%-45s  %s\n" "─────────────────────────────────────────────" "────────"

for parser in "${PARSERS[@]}"; do
	for input in "${SCHEMA_INPUTS[@]}"; do
		json="${CRITERION_DIR}/compare_schema_parse/${parser}/${input}/new/estimates.json"
		pct=$(read_ci_pct "$json")
		printf "%-45s  %s%%\n" "schema/${parser}/${input}" "$pct"
	done
done

for parser in "${PARSERS[@]}"; do
	for input in "${EXEC_INPUTS[@]}"; do
		json="${CRITERION_DIR}/compare_executable_parse/${parser}/${input}/new/estimates.json"
		pct=$(read_ci_pct "$json")
		printf "%-45s  %s%%\n" "exec/${parser}/${input}" "$pct"
	done
done

for name in "${LEXER_NAMES[@]}"; do
	dir=$(find_lexer_dir "$name")
	if [ -n "$dir" ]; then
		json="${dir}/new/estimates.json"
		pct=$(read_ci_pct "$json")
		printf "%-45s  %s%%\n" "lexer/${name}" "$pct"
	fi
done

echo ""
