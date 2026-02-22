import argparse
import re
import subprocess
import sys
from pathlib import Path

CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"
DEFAULT_EXPECTED_PATTERNS = [
    "youtube",
    "photos",
    "play store",
    "playstore",
    "vending",
    "calendar",
]
DEFAULT_OPTIONAL_PATTERNS = ["squalr", "com.squalr.android"]
PROCESS_LINE_REGEX = re.compile(r"process_id:\s*(?P<process_id>\d+),\s*name:\s*(?P<process_name>.*?),\s*is_windowed:\s*(?P<is_windowed>true|false)")
SU_INVOCATION_ATTEMPTS = [
    ("su -c", ["su", "-c"]),
    ("su 0 sh -c", ["su", "0", "sh", "-c"]),
    ("su root sh -c", ["su", "root", "sh", "-c"]),
]


def run_command(command_segments, working_directory):
    command_display = " ".join(command_segments)
    print(f"\n> {command_display}")
    command_process = subprocess.run(
        command_segments,
        cwd=working_directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    command_output = command_process.stdout or ""
    if command_output:
        print(command_output, end="")
    return command_process.returncode, command_output


def fail(error_message):
    print(f"\nERROR: {error_message}")
    sys.exit(1)


def ensure_device_connected(working_directory):
    exit_code, devices_output = run_command(["adb", "devices"], working_directory)
    if exit_code != 0:
        fail("Failed to query adb devices.")

    device_output_lines = [output_line.strip() for output_line in devices_output.splitlines() if output_line.strip()]
    has_connected_device = any(output_line.endswith("\tdevice") for output_line in device_output_lines[1:])
    if not has_connected_device:
        fail("No connected adb device found.")


def run_su_command_with_fallback(working_directory, shell_command, action_label):
    for su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
        exit_code, command_output = run_command(["adb", "shell", *su_invocation_prefix, shell_command], working_directory)
        if exit_code == 0:
            print(f"Succeeded: {action_label} via {su_invocation_label}")
            return command_output

    fail(
        f"Failed to {action_label} with all su invocation attempts: "
        + ", ".join(invocation_label for invocation_label, _ in SU_INVOCATION_ATTEMPTS)
    )


def deploy_if_requested(working_directory, should_deploy, deploy_release_mode):
    if not should_deploy:
        return

    deploy_command_segments = ["python", "./build_and_deploy.py"]
    deploy_command_segments.append("--release" if deploy_release_mode else "--debug")
    deploy_exit_code, _ = run_command(deploy_command_segments, working_directory)
    if deploy_exit_code != 0:
        fail("Android deploy failed.")


def parse_windowed_process_names(cli_output_text):
    discovered_process_names = []

    for output_line in cli_output_text.splitlines():
        regex_match = PROCESS_LINE_REGEX.search(output_line)
        if not regex_match:
            continue

        if regex_match.group("is_windowed") != "true":
            continue

        process_name = regex_match.group("process_name").strip()
        if process_name:
            discovered_process_names.append(process_name)

    return sorted(set(discovered_process_names), key=str.lower)


def classify_processes(discovered_process_names, required_patterns, optional_patterns):
    missing_required_patterns = []
    for required_pattern in required_patterns:
        required_pattern_normalized = required_pattern.casefold()
        if not any(required_pattern_normalized in process_name.casefold() for process_name in discovered_process_names):
            missing_required_patterns.append(required_pattern)

    unexpected_process_names = []
    for process_name in discovered_process_names:
        process_name_normalized = process_name.casefold()
        is_required_match = any(required_pattern.casefold() in process_name_normalized for required_pattern in required_patterns)
        is_optional_match = any(optional_pattern.casefold() in process_name_normalized for optional_pattern in optional_patterns)
        if not is_required_match and not is_optional_match:
            unexpected_process_names.append(process_name)

    return missing_required_patterns, unexpected_process_names


def main():
    argument_parser = argparse.ArgumentParser(
        description=(
            "Verify Android windowed process quality by checking `squalr-cli process list -w` output "
            "against expected app-name patterns."
        )
    )
    argument_parser.add_argument(
        "--deploy",
        action="store_true",
        help="Run `build_and_deploy.py` before verification.",
    )
    argument_parser.add_argument(
        "--release",
        action="store_true",
        help="Use `--release` when `--deploy` is set. Defaults to debug deploy.",
    )
    argument_parser.add_argument(
        "--limit",
        type=int,
        default=300,
        help="CLI process list limit passed to `squalr-cli process list -w -l`.",
    )
    argument_parser.add_argument(
        "--expected-pattern",
        action="append",
        dest="expected_patterns",
        help=(
            "Required case-insensitive substring expected in the process names. "
            "Can be passed multiple times."
        ),
    )
    argument_parser.add_argument(
        "--optional-pattern",
        action="append",
        dest="optional_patterns",
        help=(
            "Allowed case-insensitive substring that does not count as required. "
            "Can be passed multiple times."
        ),
    )
    parsed_arguments = argument_parser.parse_args()

    workspace_directory = Path(__file__).resolve().parent
    ensure_device_connected(workspace_directory)

    required_patterns = parsed_arguments.expected_patterns or list(DEFAULT_EXPECTED_PATTERNS)
    optional_patterns = parsed_arguments.optional_patterns or list(DEFAULT_OPTIONAL_PATTERNS)

    deploy_if_requested(workspace_directory, parsed_arguments.deploy, parsed_arguments.release)

    cli_command = f"{CLI_DEVICE_PATH} process list -w -l {parsed_arguments.limit}"
    cli_output_text = run_su_command_with_fallback(workspace_directory, cli_command, "query windowed process list")
    discovered_process_names = parse_windowed_process_names(cli_output_text)

    print("\nDiscovered windowed process names:")
    if discovered_process_names:
        for process_name in discovered_process_names:
            print(f"- {process_name}")
    else:
        print("- <none>")

    missing_required_patterns, unexpected_process_names = classify_processes(
        discovered_process_names,
        required_patterns,
        optional_patterns,
    )

    if missing_required_patterns or unexpected_process_names:
        print("\nVerification failed.")
        if missing_required_patterns:
            print(f"Missing required patterns: {', '.join(missing_required_patterns)}")
        if unexpected_process_names:
            print("Unexpected process names:")
            for process_name in unexpected_process_names:
                print(f"- {process_name}")
        sys.exit(2)

    print("\nVerification passed: all required patterns were present and no unexpected process names were found.")


if __name__ == "__main__":
    main()