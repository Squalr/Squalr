#!/usr/bin/env sh
set -eu

script_directory_path="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
apk_file_path="${script_directory_path}/squalr.apk"
worker_binary_file_path="${script_directory_path}/squalr-cli"
worker_device_path="/data/local/tmp/squalr-cli"

if ! command -v adb >/dev/null 2>&1; then
  echo "Missing required command: adb"
  exit 1
fi

if [ ! -f "${apk_file_path}" ]; then
  echo "Missing APK artifact: ${apk_file_path}"
  exit 1
fi

if [ ! -f "${worker_binary_file_path}" ]; then
  echo "Missing worker artifact: ${worker_binary_file_path}"
  exit 1
fi

echo "Installing APK..."
adb install -r "${apk_file_path}"

echo "Pushing worker binary..."
adb push "${worker_binary_file_path}" "${worker_device_path}"

if adb shell su -c "chmod +x ${worker_device_path}"; then
  echo "Marked worker binary executable via su -c."
elif adb shell su 0 sh -c "chmod +x ${worker_device_path}"; then
  echo "Marked worker binary executable via su 0 sh -c."
elif adb shell su root sh -c "chmod +x ${worker_device_path}"; then
  echo "Marked worker binary executable via su root sh -c."
else
  echo "Failed to chmod worker binary with supported su invocations."
  exit 1
fi

echo "Install complete."
