#!/bin/bash

# Runs fuzz tests for the libgraphql-parser crate using cargo-fuzz.
#
# Usage:
#   ./crates/libgraphql-parser/scripts/run-fuzz-tests.sh
#   ./crates/libgraphql-parser/scripts/run-fuzz-tests.sh 15
#   ./crates/libgraphql-parser/scripts/run-fuzz-tests.sh 1 fuzz_lexer
#
# Arguments:
#   $1 - Duration in minutes per target (default: 1min)
#   $2 - Target name (default: all targets, run in parallel)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

FUZZ_DIR="$(to_absolute_path ".." "${SCRIPT_DIR}")"
DURATION_MINS="${1:-1}"
DURATION_SECS=$((DURATION_MINS * 60))
TARGET="${2:-}"

ALL_TARGETS=(
	fuzz_lexer
	fuzz_parse_schema
	fuzz_parse_executable
	fuzz_parse_mixed
)

assert_cargo_installed "cargo-fuzz" || exit 1

run_fuzz_target() {
	local target="$1"
	local duration_secs="$2"
	local duration_mins="$3"
	local log_file="${4:-}"

	# When running in parallel, redirect to log file
	if [ -n "${log_file}" ]; then
		exec > "${log_file}" 2>&1
	fi

	echo "────────────────────────────────────────────────"
	echo "  Fuzzing: ${target} (${duration_mins}m)"
	echo "────────────────────────────────────────────────"

	# First positional arg = main corpus dir (read-write, per-target)
	# Second positional arg = seed corpus dir (read-only, shared)
	cd "${FUZZ_DIR}/fuzz"
	cargo +nightly fuzz run "${target}" \
		"${FUZZ_DIR}/fuzz/corpus/${target}" \
		"${FUZZ_DIR}/fuzz/seed_corpus" \
		-- \
		-max_total_time="${duration_secs}"

	local status=$?
	if [ $status -eq 0 ]; then
		echo "${UNICODE_GREEN_CHECK} ${target}: passed (${duration_mins}m, no crashes)"
	else
		echo "${UNICODE_RED_X} ${target}: CRASHED (exit code ${status})"
		return $status
	fi
}

if [ -n "${TARGET}" ]; then
	# Single target: run directly (no parallelism)
	run_fuzz_target "${TARGET}" "${DURATION_SECS}" "${DURATION_MINS}"
else
	echo ""
	echo "Running all ${#ALL_TARGETS[@]} fuzz targets in parallel (${DURATION_MINS}m each)..."
	echo ""

	# Create temp directory for per-target log files
	TMPDIR_FUZZ="$(mktemp -d)"

	# Build fuzz targets once before launching parallel runs
	cd "${FUZZ_DIR}/fuzz"
	cargo +nightly fuzz build

	# Launch all targets in parallel as background subshells.
	# Use indexed arrays (bash 3.2 compatible -- no declare -A).
	PIDS=()
	for i in "${!ALL_TARGETS[@]}"; do
		target="${ALL_TARGETS[$i]}"
		local_log="${TMPDIR_FUZZ}/${target}.log"
		(
			run_fuzz_target "${target}" "${DURATION_SECS}" "${DURATION_MINS}" "${local_log}"
		) &
		PIDS[$i]=$!
	done

	# Ensure Ctrl+C kills background fuzz processes and cleans up
	cleanup() {
		for pid in "${PIDS[@]}"; do
			kill "${pid}" 2>/dev/null
		done
		wait 2>/dev/null
		rm -rf "${TMPDIR_FUZZ}"
	}
	trap cleanup EXIT

	# Monitor targets — emit output immediately as each finishes.
	# Crashes are highlighted so the user can Ctrl+C to address them
	# while other targets are still running.
	FINISHED=()
	FAILED=0
	for i in "${!ALL_TARGETS[@]}"; do
		FINISHED[$i]=0
	done

	while true; do
		all_done=true
		for i in "${!ALL_TARGETS[@]}"; do
			# Skip already-reaped processes
			if [ "${FINISHED[$i]}" -eq 1 ]; then
				continue
			fi

			# Check if this process is still running
			if kill -0 "${PIDS[$i]}" 2>/dev/null; then
				all_done=false
			else
				# Process exited — reap it and get exit code
				set +e
				wait "${PIDS[$i]}"
				exit_code=$?
				set -e
				FINISHED[$i]=1

				target="${ALL_TARGETS[$i]}"
				local_log="${TMPDIR_FUZZ}/${target}.log"

				if [ "${exit_code}" -ne 0 ]; then
					# Print crash output immediately
					echo ""
					echo "╔════════════════════════════════════════════════╗"
					echo "║  ${UNICODE_RED_X} CRASH: ${target}"
					echo "╚════════════════════════════════════════════════╝"
					if [ -f "${local_log}" ]; then
						cat "${local_log}"
					fi
					echo ""
					echo "  (Other targets still running — Ctrl+C to stop)"
					echo ""
					FAILED=$((FAILED + 1))
				else
					# Print passing output
					if [ -f "${local_log}" ]; then
						cat "${local_log}"
					fi
					echo ""
				fi
			fi
		done

		if ${all_done}; then
			break
		fi

		sleep 1
	done

	echo "════════════════════════════════════════════════"
	if [ $FAILED -eq 0 ]; then
		echo "${UNICODE_GREEN_CHECK} All ${#ALL_TARGETS[@]} targets passed"
	else
		echo "${UNICODE_RED_X} ${FAILED}/${#ALL_TARGETS[@]} targets crashed"
		exit 1
	fi
fi
