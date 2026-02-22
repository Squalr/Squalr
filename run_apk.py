import argparse
import subprocess
import sys
import time
from pathlib import Path


PACKAGE_CANDIDATES = ["com.squalr.android"]
LEGACY_PACKAGE = "rust.squalr_android"
MAIN_ACTIVITY_NAME = "android.app.NativeActivity"


def run_command(command_segments):
    """Run a command and return the exit code plus captured output."""
    command_display = " ".join(command_segments)
    print(f"\n> {command_display}")
    process = subprocess.run(
        command_segments,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    output_text = process.stdout or ""
    if output_text:
        print(output_text, end="")
    return process.returncode, output_text


def ensure_device_connected():
    exit_code, output_text = run_command(["adb", "devices"])
    if exit_code != 0:
        print("Failed to query adb devices.")
        sys.exit(exit_code)

    output_lines = [output_line.strip() for output_line in output_text.splitlines() if output_line.strip()]
    has_device = any(output_line.endswith("\tdevice") for output_line in output_lines[1:])
    if not has_device:
        print("No connected adb device found.")
        sys.exit(1)


def resolve_launch_activity(package_name):
    exit_code, output_text = run_command(
        [
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
            package_name,
        ]
    )
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
    if not component_name.startswith(f"{package_name}/"):
        return None
    return component_name


def launch_known_main_activity(package_name):
    explicit_component_name = f"{package_name}/{MAIN_ACTIVITY_NAME}"
    exit_code, _ = run_command(["adb", "shell", "am", "start", "-n", explicit_component_name])
    if exit_code == 0:
        return explicit_component_name
    return None


def launch_component(component_name):
    exit_code, _ = run_command(["adb", "shell", "am", "start", "-n", component_name])
    if exit_code != 0:
        print(f"Failed to launch component: {component_name}")
        sys.exit(exit_code)


def prepare_launch_diagnostics(package_name):
    run_command(["adb", "shell", "am", "force-stop", package_name])
    run_command(["adb", "logcat", "-c"])


def collect_launch_diagnostics(package_name, launch_log_seconds, launch_log_file_path):
    print(f"\nCollecting launch diagnostics for {launch_log_seconds} second(s)...")
    time.sleep(launch_log_seconds)

    run_command(["adb", "shell", "pidof", package_name])
    run_command(["adb", "shell", "dumpsys", "activity", "activities", package_name])
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
        ]
    )

    if launch_log_file_path:
        launch_log_file_path.parent.mkdir(parents=True, exist_ok=True)
        launch_log_file_path.write_text(logcat_output, encoding="utf-8")
        print(f"\nSaved launch logcat to: {launch_log_file_path}")


def main():
    argument_parser = argparse.ArgumentParser(description="Launch installed Squalr Android app over adb.")
    argument_parser.add_argument(
        "--package",
        help="Package name override. Defaults to checking known Squalr package ids.",
    )
    argument_parser.add_argument(
        "--include-legacy-package",
        action="store_true",
        help="Also try rust.squalr_android as a fallback for older installs.",
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

    ensure_device_connected()
    launch_log_file_path = Path(parsed_arguments.launch_log_file).resolve() if parsed_arguments.launch_log_file else None

    if parsed_arguments.package:
        package_candidates = [parsed_arguments.package]
    else:
        package_candidates = list(PACKAGE_CANDIDATES)
        if parsed_arguments.include_legacy_package:
            package_candidates.append(LEGACY_PACKAGE)
    for package_name in package_candidates:
        prepare_launch_diagnostics(package_name)

        launched_component_name = launch_known_main_activity(package_name)
        if launched_component_name is not None:
            collect_launch_diagnostics(package_name, parsed_arguments.launch_log_seconds, launch_log_file_path)
            print(f"\nLaunched: {launched_component_name}")
            return

        resolved_component_name = resolve_launch_activity(package_name)
        if resolved_component_name is None:
            continue

        launch_component(resolved_component_name)
        collect_launch_diagnostics(package_name, parsed_arguments.launch_log_seconds, launch_log_file_path)
        print(f"\nLaunched: {resolved_component_name}")
        return

    package_list_display = ", ".join(package_candidates)
    print(f"Could not resolve a launchable activity for package(s): {package_list_display}")
    print("Install the APK first with build_and_deploy.py.")
    sys.exit(1)


if __name__ == "__main__":
    main()
