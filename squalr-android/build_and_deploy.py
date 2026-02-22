import argparse
import os
import subprocess
import sys
from pathlib import Path


TARGET_TRIPLE = "aarch64-linux-android"
CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"


def run_command(command_segments, working_directory):
    """Run a command and stream output to stdout/stderr."""
    command_display = " ".join(command_segments)
    print(f"\n> {command_display}")
    process = subprocess.Popen(
        command_segments,
        cwd=working_directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    output_lines = []
    assert process.stdout is not None
    for output_line in process.stdout:
        output_lines.append(output_line)
        print(output_line, end="")
    process.wait()
    return process.returncode, "".join(output_lines)


def build_cli_binary(workspace_directory, is_release):
    cli_build_command = [
        "cargo",
        "ndk",
        "--target",
        TARGET_TRIPLE,
        "build",
        "--bin",
        "squalr-cli",
        "-v",
    ]
    if is_release:
        cli_build_command.append("--release")

    exit_code, _ = run_command(cli_build_command, workspace_directory)
    if exit_code != 0:
        print("Failed to build Android privileged worker binary.")
        sys.exit(exit_code)


def build_apk_with_fallback(android_manifest_directory, prefer_release):
    if prefer_release:
        release_apk_build_command = [
            "cargo",
            "apk",
            "build",
            "--target",
            TARGET_TRIPLE,
            "--release",
            "--lib",
        ]
        exit_code, release_output = run_command(release_apk_build_command, android_manifest_directory)
        if exit_code == 0:
            return "release"

        if "Configure a release keystore" not in release_output:
            print("Failed to build release APK for a reason other than signing configuration.")
            sys.exit(exit_code)

        print("\nRelease signing is not configured. Falling back to debug APK build.")

    debug_apk_build_command = [
        "cargo",
        "apk",
        "build",
        "--target",
        TARGET_TRIPLE,
        "--lib",
    ]
    exit_code, _ = run_command(debug_apk_build_command, android_manifest_directory)
    if exit_code != 0:
        print("Failed to build debug APK.")
        sys.exit(exit_code)

    return "debug"


def install_apk(workspace_directory, apk_profile):
    apk_path = workspace_directory / "target" / TARGET_TRIPLE / apk_profile / "apk" / "squalr-android.apk"
    if not apk_path.exists():
        print(f"Built APK not found at expected path: {apk_path}")
        sys.exit(1)

    install_command = ["adb", "install", "-r", str(apk_path)]
    exit_code, _ = run_command(install_command, workspace_directory)
    if exit_code != 0:
        print("Failed to install APK via adb.")
        sys.exit(exit_code)


def deploy_privileged_worker(workspace_directory, worker_profile):
    cli_host_path = workspace_directory / "target" / TARGET_TRIPLE / worker_profile / "squalr-cli"
    if not cli_host_path.exists():
        print(f"Built CLI binary not found at expected path: {cli_host_path}")
        sys.exit(1)

    push_command = ["adb", "push", str(cli_host_path), CLI_DEVICE_PATH]
    exit_code, _ = run_command(push_command, workspace_directory)
    if exit_code != 0:
        print("Failed to push privileged worker to device.")
        sys.exit(exit_code)

    chmod_command = ["adb", "shell", f"su -c 'chmod +x {CLI_DEVICE_PATH}'"]
    exit_code, _ = run_command(chmod_command, workspace_directory)
    if exit_code != 0:
        print("Failed to mark privileged worker as executable via su.")
        sys.exit(exit_code)

    verify_command = ["adb", "shell", f"su -c '{CLI_DEVICE_PATH} --help'"]
    exit_code, _ = run_command(verify_command, workspace_directory)
    if exit_code != 0:
        print("Failed to verify privileged worker launch via su.")
        sys.exit(exit_code)


def main():
    argument_parser = argparse.ArgumentParser(description="Build and deploy Squalr Android GUI + privileged worker.")
    argument_parser.add_argument(
        "--release",
        action="store_true",
        help="Prefer release builds. If APK signing is not configured, APK build falls back to debug.",
    )
    parsed_arguments = argument_parser.parse_args()

    script_directory = Path(__file__).resolve().parent
    workspace_directory = script_directory.parent

    build_cli_binary(workspace_directory, parsed_arguments.release)
    apk_profile = build_apk_with_fallback(script_directory, parsed_arguments.release)
    install_apk(workspace_directory, apk_profile)
    deploy_privileged_worker(workspace_directory, "release" if parsed_arguments.release else "debug")

    print("\nDeployment complete. Launch the Squalr app on device.")


if __name__ == "__main__":
    main()
