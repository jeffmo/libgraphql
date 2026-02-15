#!/bin/bash

# Records the sha256 checksum of the locally-fetched Shopify Admin
# GraphQL API schema fixture. The resulting .sha256 file is intended
# to be checked in to the repository so that future fetches can be
# verified for integrity.
#
# Prerequisites: Run the fetch script first so the schema file exists.
#
# Usage:
#   ./crates/libgraphql-parser/scripts/record-shopify-admin-graphql-schema-fixture-checksum.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
source "${REPO_ROOT}/scripts/_include.sh"

# ─── Settings ─────────────────────────────────────────────

SCHEMA_FILE="${REPO_ROOT}/crates/libgraphql-parser/benches/fixtures/third-party/shopify-admin-schema/schema.graphql"
CHECKSUM_FILE="${SCHEMA_FILE}.sha256"

# ─── Validate ─────────────────────────────────────────────

if [ ! -f "${SCHEMA_FILE}" ]; then
	{
		echo "${UNICODE_RED_X} Schema file not found at:"
		echo ""
		echo "    ${SCHEMA_FILE}"
		echo ""
		echo "Run the fetch script first:"
		echo ""
		echo "    ./crates/libgraphql-parser/scripts/fetch-shopify-admin-graphql-schema-fixture.sh"
		echo ""
	} >&2
	exit 1
fi

# ─── Record checksum ─────────────────────────────────────

HASH="$(sha256_hash "${SCHEMA_FILE}")"
echo -n "${HASH}" > "${CHECKSUM_FILE}"

echo "${UNICODE_GREEN_CHECK} Recorded sha256 checksum to:"
echo ""
echo "    ${CHECKSUM_FILE}"
echo ""
echo "  hash: ${HASH}"
echo ""
