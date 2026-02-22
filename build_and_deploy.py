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
MAIN_ACTIVITY_NAME = "android.app.NativeActivity"
ANDROID_MANIFEST_CRATE_NAME = "squalr"
SU_INVOCATION_ATTEMPTS = [
    ("su -c", ["su", "-c"]),
    ("su 0 sh -c", ["su", "0", "sh", "-c"]),
    ("su root sh -c", ["su", "root", "sh", "-c"]),
]


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


def run_su_command_with_fallback(workspace_directory, shell_command, action_label):
    for su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
        command_segments = ["adb", "shell", *su_invocation_prefix, shell_command]
        exit_code, _ = run_command(command_segments, workspace_directory)
        if exit_code == 0:
            print(f"Succeeded: {action_label} via {su_invocation_label}")
            return

    fail(
        f"Failed to {action_label} with all su invocation attempts: "
        + ", ".join(invocation_label for invocation_label, _ in SU_INVOCATION_ATTEMPTS)
    )


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

    clang_candidates_from_path = [
        "aarch64-linux-android-clang",
        "aarch64-linux-android-clang.cmd",
        "aarch64-linux-android21-clang",
        "aarch64-linux-android21-clang.cmd",
    ]
    clang_from_path = any(command_exists(clang_candidate) for clang_candidate in clang_candidates_from_path)
    clang_from_ndk = list(ndk_root_path.glob("toolchains/llvm/prebuilt/*/bin/aarch64-linux-android*-clang*"))
    legacy_clang_from_ndk = list(ndk_root_path.glob("build/core/toolchains/aarch64-linux-android-clang*"))
    if not clang_from_path and not clang_from_ndk and not legacy_clang_from_ndk:
        fail(
            "Could not find Android clang toolchain in PATH or under ANDROID_NDK_ROOT "
            "(expected aarch64-linux-android*-clang in toolchains/llvm/prebuilt/*/bin)."
        )

    cargo_ndk_version_command = ["cargo", "ndk", "--version"]
    exit_code, _ = run_command(cargo_ndk_version_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to run `cargo ndk --version`. Install with: cargo install cargo-ndk")

    cargo_apk_probe_command = ["cargo", "apk", "--help"]
    exit_code, _ = run_command(cargo_apk_probe_command, workspace_directory)
    if exit_code != 0:
        fail("Failed to run `cargo apk --help`. Install with: cargo install cargo-apk")


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
        workspace_directory / "target" / TARGET_TRIPLE / apk_profile / "apk" / f"{ANDROID_MANIFEST_CRATE_NAME}.apk",
        workspace_directory / "target" / apk_profile / "apk" / f"{ANDROID_MANIFEST_CRATE_NAME}.apk",
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

    run_su_command_with_fallback(
        workspace_directory,
        f"chmod +x {CLI_DEVICE_PATH}",
        "mark privileged worker as executable",
    )
    run_su_command_with_fallback(
        workspace_directory,
        f"{CLI_DEVICE_PATH} --help",
        "verify privileged worker launch",
    )


def resolve_launch_activity(workspace_directory):
    resolve_command = [
        "adb",
        "shell",
        "cmd",
        "package",
        "resolve-activity",
        "--brief",
        "-a",
        "android.intent.action.MAIN",
        "-c",
        "android.intent.category.LAUNCHER",
        PACKAGE_NAME,
    ]
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
    if not component_name.startswith(f"{PACKAGE_NAME}/"):
        return None

    return component_name


def launch_installed_app(workspace_directory):
    explicit_component_name = f"{PACKAGE_NAME}/{MAIN_ACTIVITY_NAME}"
    explicit_launch_command = ["adb", "shell", "am", "start", "-n", explicit_component_name]
    explicit_launch_exit_code, _ = run_command(explicit_launch_command, workspace_directory)
    if explicit_launch_exit_code == 0:
        return

    resolved_component_name = resolve_launch_activity(workspace_directory)
    if resolved_component_name is None:
        fail(f"Could not resolve launchable activity for package: {PACKAGE_NAME}")

    resolved_launch_command = ["adb", "shell", "am", "start", "-n", resolved_component_name]
    resolved_launch_exit_code, _ = run_command(resolved_launch_command, workspace_directory)
    if resolved_launch_exit_code != 0:
        fail("Failed to launch the installed APK.")


def verify_launcher_identity(workspace_directory):
    resolved_component_name = resolve_launch_activity(workspace_directory)
    if resolved_component_name is None:
        fail(f"Could not resolve launchable activity for package: {PACKAGE_NAME}")

    expected_component_name = f"{PACKAGE_NAME}/{MAIN_ACTIVITY_NAME}"
    if resolved_component_name != expected_component_name:
        fail(
            "Launcher identity mismatch. "
            f"Expected {expected_component_name}, got {resolved_component_name}"
        )


def prepare_launch_diagnostics(workspace_directory):
    run_command(["adb", "shell", "am", "force-stop", PACKAGE_NAME], workspace_directory)
    run_command(["adb", "logcat", "-c"], workspace_directory)


def summarize_activity_draw_state(activity_dump_output):
    draw_state_lines = [
        output_line.strip()
        for output_line in activity_dump_output.splitlines()
        if PACKAGE_NAME in output_line and "reportedDrawn=" in output_line
    ]
    if not draw_state_lines:
        print("No reportedDrawn activity state line was found in dumpsys activity output.")
        return

    print(f"Activity draw state: {draw_state_lines[-1]}")


def summarize_window_splash_state(window_dump_output):
    splash_lines = [
        output_line.strip()
        for output_line in window_dump_output.splitlines()
        if "Splash Screen" in output_line and PACKAGE_NAME in output_line
    ]
    if splash_lines:
        print(f"Splash window still present: {splash_lines[-1]}")
    else:
        print("No splash window entry found in dumpsys window output.")


def summarize_bootstrap_breadcrumbs(logcat_output):
    breadcrumb_lines = [
        output_line.strip()
        for output_line in logcat_output.splitlines()
        if "[android_bootstrap]" in output_line
    ]
    if not breadcrumb_lines:
        print("No Android bootstrap breadcrumbs were found in filtered logcat output.")
        return

    expected_breadcrumb_messages = [
        "Before SqualrEngine::new.",
        "After SqualrEngine::new.",
        "Before App::new.",
        "After App::new.",
        "Before first frame submission.",
    ]
    reached_breadcrumb_messages = {
        expected_breadcrumb_message
        for expected_breadcrumb_message in expected_breadcrumb_messages
        if any(expected_breadcrumb_message in breadcrumb_line for breadcrumb_line in breadcrumb_lines)
    }
    missing_breadcrumb_messages = [
        expected_breadcrumb_message
        for expected_breadcrumb_message in expected_breadcrumb_messages
        if expected_breadcrumb_message not in reached_breadcrumb_messages
    ]

    print(f"Last Android bootstrap breadcrumb: {breadcrumb_lines[-1]}")
    if missing_breadcrumb_messages:
        print(f"Missing expected breadcrumbs: {', '.join(missing_breadcrumb_messages)}")
    else:
        print("Reached all expected Android bootstrap breadcrumbs through first frame submission.")


def collect_launch_diagnostics(workspace_directory, launch_log_seconds, launch_log_file_path):
    print(f"\nCollecting launch diagnostics for {launch_log_seconds} second(s)...")
    time.sleep(launch_log_seconds)

    run_command(["adb", "shell", "pidof", PACKAGE_NAME], workspace_directory)
    _, activity_dump_output = run_command(["adb", "shell", "dumpsys", "activity", "activities", PACKAGE_NAME], workspace_directory)
    summarize_activity_draw_state(activity_dump_output)
    _, window_dump_output = run_command(["adb", "shell", "dumpsys", "window", "windows"], workspace_directory)
    summarize_window_splash_state(window_dump_output)
    _, logcat_output = run_command(
        [
            "adb",
            "logcat",
            "-d",
            "-v",
            "threadtime",
            "Squalr:I",
            "ActivityTaskManager:I",
            "ActivityManager:I",
            "AndroidRuntime:E",
            "DEBUG:E",
            "libc:E",
            "*:S",
        ],
        workspace_directory,
    )
    summarize_bootstrap_breadcrumbs(logcat_output)

    if launch_log_file_path:
        launch_log_file_path.parent.mkdir(parents=True, exist_ok=True)
        launch_log_file_path.write_text(logcat_output, encoding="utf-8")
        print(f"\nSaved launch logcat to: {launch_log_file_path}")


def verify_ipc_handshake(workspace_directory):
    print("\nWaiting for privileged worker IPC shell to come online...")
    for _handshake_poll_attempt in range(12):
        for _su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
            poll_command = ["adb", "shell", *su_invocation_prefix, "pidof squalr-cli"]
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
    argument_parser.add_argument(
        "--skip-worker",
        action="store_true",
        help="Skip privileged worker deployment and IPC validation (useful for launch diagnostics on non-rooted devices).",
    )
    argument_parser.add_argument(
        "--launch-log-seconds",
        type=int,
        default=6,
        help="Seconds to wait after launch before collecting log diagnostics.",
    )
    argument_parser.add_argument(
        "--launch-log-file",
        help="Optional path to write filtered launch logcat output.",
    )
    parsed_arguments = argument_parser.parse_args()

    workspace_directory = Path(__file__).resolve().parent
    android_manifest_directory = workspace_directory / ANDROID_MANIFEST_CRATE_NAME

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
    apk_profile = build_apk_with_fallback(android_manifest_directory, prefer_release_mode)

    if parsed_arguments.compile_check:
        print("\nCompile check complete.")
        return

    ensure_adb_device_connected(workspace_directory)
    install_apk(workspace_directory, apk_profile)
    verify_launcher_identity(workspace_directory)
    if not parsed_arguments.skip_worker:
        deploy_privileged_worker(workspace_directory, "release" if prefer_release_mode else "debug")
    else:
        print("\nSkipping privileged worker deployment (--skip-worker).")

    launch_log_file_path = Path(parsed_arguments.launch_log_file).resolve() if parsed_arguments.launch_log_file else None
    prepare_launch_diagnostics(workspace_directory)
    launch_installed_app(workspace_directory)
    collect_launch_diagnostics(workspace_directory, parsed_arguments.launch_log_seconds, launch_log_file_path)

    if not parsed_arguments.skip_worker:
        verify_ipc_handshake(workspace_directory)
    else:
        print("\nSkipped IPC handshake validation because --skip-worker was provided.")

    print("\nDeployment + smoke validation complete.")


if __name__ == "__main__":
    main()
