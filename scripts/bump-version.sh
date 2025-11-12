#!/bin/bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "${SCRIPT_DIR}/_include.sh"

if ! assert_installed jq; then
  exit 1
fi

if ! assert_cargo_installed "cargo-edit"; then
  exit 1
fi

CRATE_NAME="${1}"
if [ -z "${CRATE_NAME}" ]; then
  {
    echo "${UNICODE_RED_X} No crate name specified. Usage:"
    echo
    echo "  ${0} libgraphql-core [patch|minor|major]"
    echo
  } >&2
  exit 1
fi

VERSION_BUMP="${2}"
if [ -n "${VERSION_BUMP}" ]; then
  if [ "${VERSION_BUMP}" != "patch" ] \
    && [ "${VERSION_BUMP}" != "major" ] \
    && [ "${VERSION_BUMP}" != "minor" ] \
    && ! [[ "${VERSION_BUMP}" =~ [0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]* ]] ; then
    {
      echo "${UNICODE_RED_X} Invalid version-bump arg: \`${VERSION_BUMP}\`. Usage:"
      echo
      echo "  ${0} ${CRATE_NAME} [patch|minor|major]"
      echo
    } >&2
    exit 1
  fi
fi

REPO_ROOT="$(get_repo_root)"
CRATE_PATH="${REPO_ROOT}/crates/${CRATE_NAME}"

if [ ! -d "${CRATE_PATH}" ]; then
  echo "${UNICODE_RED_X} No crate found at \`${CRATE_PATH}\`!" >&2
  exit 1
fi

CARGO_TOML_FILE="${CRATE_PATH}/Cargo.toml"
if [ ! -f "${CARGO_TOML_FILE}" ]; then
  echo "${UNICODE_RED_X} No Cargo.toml file found in \`${CRATE_PATH}\`!" >&2
  exit 1
fi

CRATE_LOCAL_VERSION=$(get_crate_local_version "${CARGO_TOML_FILE}")
CRATE_PUBLISHED_VERSION=$(get_crate_published_version "${CRATE_NAME}")
echo "Current local crate version: \`${CRATE_LOCAL_VERSION}\`"
echo "Current published crate version: \`${CRATE_PUBLISHED_VERSION}\`"
echo

if [ -z "${VERSION_BUMP}" ]; then
  read -p "Enter new crate version: " NEW_CRATE_VERSION
  set -x
  cargo set-version \
    --package "${CRATE_NAME}" \
    "${NEW_CRATE_VERSION}"
elif [ "${VERSION_BUMP}" == "patch" ] \
  || [ "${VERSION_BUMP}" == "minor" ] \
  || [ "${VERSION_BUMP}" == "major" ]; then
  cargo set-version \
    --manifest-path "${CARGO_TOML_FILE}" \
    --bump "${VERSION_BUMP}"

  NEW_CRATE_VERSION=$(get_crate_local_version "${CARGO_TOML_FILE}")
else
  cargo set-version \
    --package "${CRATE_NAME}" \
    "${VERSION_BUMP}"
fi

OLD_CRATE_LOCAL_VERSION="${CRATE_LOCAL_VERSION}"
NEW_CRATE_LOCAL_VERSION="$(get_crate_local_version "${CARGO_TOML_FILE}")"

echo
echo "${UNICODE_GREEN_CHECK} Updated \`${CRATE_NAME}\` from \`${OLD_CRATE_LOCAL_VERSION}\` -> \`${NEW_CRATE_LOCAL_VERSION}\`."

COMMIT_MESSAGE="[${CRATE_NAME}][Cargo.toml] ${OLD_CRATE_LOCAL_VERSION} -> ${NEW_CRATE_LOCAL_VERSION}"

if is_installed "pbcopy"; then
  echo
  echo "  ${COMMIT_MESSAGE}"
  echo
  printf "%s" "${COMMIT_MESSAGE}" | pbcopy
else
  echo
  echo "  ${COMMIT_MESSAGE}"
  echo
fi
