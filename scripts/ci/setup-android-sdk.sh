#!/usr/bin/env bash

set -euo pipefail

resolve_android_sdk_root() {
  if [[ -n "${ANDROID_SDK_ROOT:-}" && -d "${ANDROID_SDK_ROOT}" ]]; then
    printf '%s\n' "${ANDROID_SDK_ROOT}"
    return 0
  fi

  if [[ -n "${ANDROID_HOME:-}" && -d "${ANDROID_HOME}" ]]; then
    printf '%s\n' "${ANDROID_HOME}"
    return 0
  fi

  local candidate_path
  for candidate_path in \
    "/usr/local/lib/android/sdk" \
    "${HOME}/.android/sdk" \
    "${HOME}/Android/Sdk"; do
    if [[ -d "${candidate_path}" ]]; then
      printf '%s\n' "${candidate_path}"
      return 0
    fi
  done

  return 1
}

resolve_sdkmanager_bin_path() {
  local android_sdk_root="$1"
  local candidate_path
  for candidate_path in \
    "${android_sdk_root}/cmdline-tools/latest/bin" \
    "${android_sdk_root}/cmdline-tools/18.0/bin" \
    "${android_sdk_root}/cmdline-tools/17.0/bin" \
    "${android_sdk_root}/cmdline-tools/16.0/bin" \
    "${android_sdk_root}/cmdline-tools/bin"; do
    if [[ -x "${candidate_path}/sdkmanager" ]]; then
      printf '%s\n' "${candidate_path}"
      return 0
    fi
  done

  return 1
}

main() {
  if [[ $# -eq 0 ]]; then
    echo "Provide at least one Android SDK package to install."
    exit 1
  fi

  : "${GITHUB_ENV:?GITHUB_ENV must be set.}"
  : "${GITHUB_PATH:?GITHUB_PATH must be set.}"

  local android_sdk_root
  android_sdk_root="$(resolve_android_sdk_root)" || {
    echo "Unable to locate the Android SDK root on this runner."
    exit 1
  }

  local sdkmanager_bin_path
  sdkmanager_bin_path="$(resolve_sdkmanager_bin_path "${android_sdk_root}")" || {
    echo "Unable to locate sdkmanager under ${android_sdk_root}."
    exit 1
  }

  {
    echo "ANDROID_HOME=${android_sdk_root}"
    echo "ANDROID_SDK_ROOT=${android_sdk_root}"
  } >> "${GITHUB_ENV}"

  {
    echo "${sdkmanager_bin_path}"
    echo "${android_sdk_root}/platform-tools"
  } >> "${GITHUB_PATH}"

  "${sdkmanager_bin_path}/sdkmanager" --sdk_root="${android_sdk_root}" --licenses < <(yes)
  "${sdkmanager_bin_path}/sdkmanager" --sdk_root="${android_sdk_root}" --install "$@"
}

main "$@"
