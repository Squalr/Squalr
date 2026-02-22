import argparse
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


TARGET_TRIPLE = "aarch64-linux-android"
CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"
PACKAGE_NAME = "com.squalr.android"


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


def fail(message):
    print(message)
    sys.exit(1)


def command_exists(command_name):
    return shutil.which(command_name) is not None


def ensure_host_preflight(workspace_directory):
    missing_commands = [command_name for command_name in ["cargo", "rustup", "adb"] if not command_exists(command_name)]
    if missing_commands:
        fail(f"Missing required command(s): {', '.join(missing_commands)}")

    rust_target_list_command = ["rustup", "target", "list", "--installed"]
    exit_code, output_text = run_command(rust_target_list_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to query installed Rust targets.")
    if TARGET_TRIPLE not in output_text.split():
        fail(f"Rust target '{TARGET_TRIPLE}' is not installed. Run: rustup target add {TARGET_TRIPLE}")

    android_home = os.environ.get("ANDROID_HOME")
    android_ndk_root = os.environ.get("ANDROID_NDK_ROOT")
    if not android_home:
        fail("ANDROID_HOME is not set.")
    if not android_ndk_root:
        fail("ANDROID_NDK_ROOT is not set.")

    if not Path(android_home).exists():
        fail(f"ANDROID_HOME path does not exist: {android_home}")
    ndk_root_path = Path(android_ndk_root)
    if not ndk_root_path.exists():
        fail(f"ANDROID_NDK_ROOT path does not exist: {android_ndk_root}")

    clang_from_path = command_exists("aarch64-linux-android-clang")
    clang_from_ndk = list(ndk_root_path.glob("toolchains/llvm/prebuilt/*/bin/aarch64-linux-android-clang"))
    if not clang_from_path and not clang_from_ndk:
        fail("Could not find aarch64-linux-android-clang in PATH or under ANDROID_NDK_ROOT toolchains.")

    cargo_ndk_version_command = ["cargo", "ndk", "--version"]
    exit_code, _ = run_command(cargo_ndk_version_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to run `cargo ndk --version`. Install with: cargo install cargo-ndk")

    cargo_apk_version_command = ["cargo", "apk", "--version"]
    exit_code, _ = run_command(cargo_apk_version_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to run `cargo apk --version`. Install with: cargo install cargo-apk")


def ensure_adb_device_connected(workspace_directory):
    exit_code, output_text = run_command(["adb", "devices"], workspace_directory)
    if exit_code != 0:
        fail("Failed to query adb devices.")

    output_lines = [output_line.strip() for output_line in output_text.splitlines() if output_line.strip()]
    has_device = any(output_line.endswith("\tdevice") for output_line in output_lines[1:])
    if not has_device:
        fail("No connected adb device found.")


def build_cli_binary(workspace_directory, is_release):
    cli_build_command = [
        "cargo",
        "ndk",
        "--target",
        TARGET_TRIPLE,
        "build",
        "-p",
        "squalr-cli",
        "-v",
    ]
    if is_release:
        cli_build_command.append("--release")

    exit_code, _ = run_command(cli_build_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to build Android privileged worker binary.")


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
            fail("Failed to build release APK for a reason other than signing configuration.")

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
        fail("Failed to build debug APK.")

    return "debug"


def install_apk(workspace_directory, apk_profile):
    apk_candidate_paths = [
        workspace_directory / "target" / TARGET_TRIPLE / apk_profile / "apk" / "squalr-android.apk",
        workspace_directory / "target" / apk_profile / "apk" / "squalr-android.apk",
    ]
    apk_path = next((candidate_path for candidate_path in apk_candidate_paths if candidate_path.exists()), None)
    if apk_path is None:
        print("Built APK not found in expected locations:")
        for candidate_path in apk_candidate_paths:
            print(f"  - {candidate_path}")
        fail("Cannot continue without an APK.")

    install_command = ["adb", "install", "-r", str(apk_path)]
    exit_code, _ = run_command(install_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to install APK via adb.")


def deploy_privileged_worker(workspace_directory, worker_profile):
    cli_host_path = workspace_directory / "target" / TARGET_TRIPLE / worker_profile / "squalr-cli"
    if not cli_host_path.exists():
        fail(f"Built CLI binary not found at expected path: {cli_host_path}")

    push_command = ["adb", "push", str(cli_host_path), CLI_DEVICE_PATH]
    exit_code, _ = run_command(push_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to push privileged worker to device.")

    chmod_command = ["adb", "shell", f"su -c 'chmod +x {CLI_DEVICE_PATH}'"]
    exit_code, _ = run_command(chmod_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to mark privileged worker as executable via su.")

    verify_command = ["adb", "shell", f"su -c '{CLI_DEVICE_PATH} --help'"]
    exit_code, _ = run_command(verify_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to verify privileged worker launch via su.")


def resolve_launch_activity(workspace_directory):
    resolve_command = ["adb", "shell", "cmd", "package", "resolve-activity", "--brief", PACKAGE_NAME]
    exit_code, output_text = run_command(resolve_command, workspace_directory)
    if exit_code != 0:
        return None

    output_lines = [output_line.strip() for output_line in output_text.splitlines() if output_line.strip()]
    if not output_lines:
        return None
    if output_lines[-1] == "No activity found":
        return None

    component_name = output_lines[-1]
    if "/" not in component_name:
        return None

    return component_name


def launch_installed_app(workspace_directory):
    component_name = resolve_launch_activity(workspace_directory)
    if component_name is None:
        fail(f"Could not resolve launchable activity for package: {PACKAGE_NAME}")

    launch_command = ["adb", "shell", "am", "start", "-n", component_name]
    exit_code, _ = run_command(launch_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to launch the installed APK.")


def verify_ipc_handshake(workspace_directory):
    print("\nWaiting for privileged worker IPC shell to come online...")
    for _poll_index in range(12):
        poll_command = ["adb", "shell", "su -c 'pidof squalr-cli'"]
        exit_code, output_text = run_command(poll_command, workspace_directory)
        if exit_code == 0 and output_text.strip():
            print("Detected running privileged worker process.")
            return
        time.sleep(1)

    fail("Privileged worker IPC handshake check failed: no running `squalr-cli` process was detected after launch.")


def main():
    argument_parser = argparse.ArgumentParser(description="Build and deploy Squalr Android GUI + privileged worker.")
    build_mode_group = argument_parser.add_mutually_exclusive_group()
    build_mode_group.add_argument(
        "--release",
        action="store_true",
        help="Use release build mode. If APK signing is not configured, APK build falls back to debug.",
    )
    build_mode_group.add_argument(
        "--debug",
        action="store_true",
        help="Use debug build mode without prompting.",
    )
    argument_parser.add_argument(
        "--compile-check",
        action="store_true",
        help="Run host preflight and Android compile checks only (no adb install/launch).",
    )
    parsed_arguments = argument_parser.parse_args()

    script_directory = Path(__file__).resolve().parent
    workspace_directory = script_directory.parent

    if parsed_arguments.release:
        prefer_release_mode = True
    elif parsed_arguments.debug:
        prefer_release_mode = False
    elif parsed_arguments.compile_check:
        prefer_release_mode = False
    else:
        release_prompt = input("Build in release mode? (y/n [default]): ").strip().lower()
        prefer_release_mode = release_prompt == "y"

    ensure_host_preflight(workspace_directory)
    build_cli_binary(workspace_directory, prefer_release_mode)
    _apk_profile = build_apk_with_fallback(script_directory, prefer_release_mode)

    if parsed_arguments.compile_check:
        print("\nCompile check complete.")
        return

    ensure_adb_device_connected(workspace_directory)
    install_apk(workspace_directory, _apk_profile)
    deploy_privileged_worker(workspace_directory, "release" if prefer_release_mode else "debug")
    launch_installed_app(workspace_directory)
    verify_ipc_handshake(workspace_directory)

    print("\nDeployment + smoke validation complete.")


if __name__ == "__main__":
    main()
