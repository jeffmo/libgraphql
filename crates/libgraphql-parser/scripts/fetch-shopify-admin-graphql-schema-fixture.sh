#!/bin/bash

# Fetches the Shopify Admin GraphQL API schema (in SDL format) from
# Shopify's public introspection proxy and writes it to the
# third-party fixtures directory. Requires `npx` (Node.js).
#
# The schema is NOT checked in to the repository (it is gitignored)
# because it is not clear whether Shopify's licensing terms permit
# vendoring it. Only its sha256 checksum is committed so that
# downloaded copies can be verified for reproducibility.
#
# Usage:
#   ./crates/libgraphql-parser/scripts/fetch-shopify-admin-graphql-schema-fixture.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

# ─── Settings ─────────────────────────────────────────────

# Pin to a specific Shopify Admin API version so that every run of
# this script fetches the exact same schema. Update this value (and
# re-record the checksum) when you want to move to a newer version.
SHOPIFY_API_VERSION="2025-10"

SHOPIFY_PROXY_URL="https://shopify.dev/admin-graphql-direct-proxy/${SHOPIFY_API_VERSION}"

OUTPUT_DIR="${REPO_ROOT}/crates/libgraphql-parser/benches/fixtures/third-party/shopify-admin-schema"
OUTPUT_FILE="${OUTPUT_DIR}/schema.graphql"
CHECKSUM_FILE="${OUTPUT_FILE}.sha256"

# ─── Prerequisites ────────────────────────────────────────

assert_installed npx || exit 1

# ─── Fetch ────────────────────────────────────────────────

echo "Fetching Shopify Admin GraphQL API schema (${SHOPIFY_API_VERSION})..."
echo "  endpoint: ${SHOPIFY_PROXY_URL}"
echo ""

mkdir -p "${OUTPUT_DIR}"

TEMP_FILE="$(mktemp)"
trap 'rm -f "${TEMP_FILE}"' EXIT

if ! npx --yes get-graphql-schema "${SHOPIFY_PROXY_URL}" > "${TEMP_FILE}"; then
	{
		echo ""
		echo "${UNICODE_RED_X} Failed to fetch schema from ${SHOPIFY_PROXY_URL}"
	} >&2
	rm -f "${TEMP_FILE}"
	exit 1
fi

if [ ! -s "${TEMP_FILE}" ]; then
	{
		echo ""
		echo "${UNICODE_RED_X} Fetched schema is empty. The introspection"
		echo "  endpoint may be unreachable or returning errors."
	} >&2
	rm -f "${TEMP_FILE}"
	exit 1
fi

mv "${TEMP_FILE}" "${OUTPUT_FILE}"
trap - EXIT

# ─── Verify checksum (if a checksum file exists) ─────────

if [ -f "${CHECKSUM_FILE}" ]; then
	EXPECTED_HASH="$(cat "${CHECKSUM_FILE}" | tr -d '[:space:]')"
	ACTUAL_HASH="$(sha256sum "${OUTPUT_FILE}" | awk '{print $1}')"

	if [ "${ACTUAL_HASH}" != "${EXPECTED_HASH}" ]; then
		{
			echo ""
			echo "${UNICODE_RED_X} Checksum mismatch!"
			echo ""
			echo "  expected: ${EXPECTED_HASH}"
			echo "  actual:   ${ACTUAL_HASH}"
			echo ""
			echo "If the schema has intentionally changed (e.g. you"
			echo "updated SHOPIFY_API_VERSION), re-record the checksum"
			echo "by running:"
			echo ""
			echo "  ${REPO_ROOT}/crates/libgraphql-parser/scripts/record-shopify-admin-graphql-schema-fixture-checksum.sh"
			echo ""
		} >&2
		exit 1
	fi

	echo "${UNICODE_GREEN_CHECK} Schema downloaded and verified (sha256 matches)."
else
	echo "${UNICODE_GREEN_CHECK} Schema downloaded to ${OUTPUT_FILE}"
	echo ""
	echo "  No checksum file found. To record one, run:"
	echo ""
	echo "    ${REPO_ROOT}/crates/libgraphql-parser/scripts/record-shopify-admin-graphql-schema-fixture-checksum.sh"
	echo ""
fi
