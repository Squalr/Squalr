import argparse
import subprocess
import sys


PACKAGE_CANDIDATES = ["com.squalr.android", "rust.squalr_android"]


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
        ["adb", "shell", "cmd", "package", "resolve-activity", "--brief", package_name]
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
    return component_name


def launch_component(component_name):
    exit_code, _ = run_command(["adb", "shell", "am", "start", "-n", component_name])
    if exit_code != 0:
        print(f"Failed to launch component: {component_name}")
        sys.exit(exit_code)


def main():
    argument_parser = argparse.ArgumentParser(description="Launch installed Squalr Android app over adb.")
    argument_parser.add_argument(
        "--package",
        help="Package name override. Defaults to checking known Squalr package ids.",
    )
    parsed_arguments = argument_parser.parse_args()

    ensure_device_connected()

    package_candidates = [parsed_arguments.package] if parsed_arguments.package else PACKAGE_CANDIDATES
    for package_name in package_candidates:
        component_name = resolve_launch_activity(package_name)
        if component_name is None:
            continue

        launch_component(component_name)
        print(f"\nLaunched: {component_name}")
        return

    package_list_display = ", ".join(package_candidates)
    print(f"Could not resolve a launchable activity for package(s): {package_list_display}")
    print("Install the APK first with build_and_deploy.py.")
    sys.exit(1)


if __name__ == "__main__":
    main()
