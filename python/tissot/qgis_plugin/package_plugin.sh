#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/dist"
PACKAGE_NAME="tissot_processing_provider"
VERSION="$(grep -E '^version=' "${SCRIPT_DIR}/metadata.txt" | cut -d'=' -f2)"
ZIP_PATH="${OUTPUT_DIR}/${PACKAGE_NAME}-${VERSION}.zip"
STAGE_DIR="${OUTPUT_DIR}/_stage"

mkdir -p "${OUTPUT_DIR}"
rm -f "${ZIP_PATH}"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/${PACKAGE_NAME}"

# Copy only runtime files into a single plugin folder, then zip that folder.
cp "${SCRIPT_DIR}/__init__.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/provider.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/xray_algorithm.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/check_algorithm.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/score_algorithm.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/diff_algorithm.py" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/metadata.txt" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/icon.png" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/README.md" "${STAGE_DIR}/${PACKAGE_NAME}/"
cp "${SCRIPT_DIR}/LICENSE" "${STAGE_DIR}/${PACKAGE_NAME}/"

(
  cd "${STAGE_DIR}"
  zip -r "${ZIP_PATH}" "${PACKAGE_NAME}"
)

rm -rf "${STAGE_DIR}"

echo "Created ${ZIP_PATH}"
