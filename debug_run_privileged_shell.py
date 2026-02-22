import argparse
import subprocess
import sys


DEFAULT_CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"


def run_command(command_segments):
    command_display = " ".join(command_segments)
    print(f"> {command_display}")
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


def fail(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def ensure_device_connected():
    exit_code, output_text = run_command(["adb", "devices"])
    if exit_code != 0:
        fail("Failed to query adb devices.")

    output_lines = [output_line.strip() for output_line in output_text.splitlines() if output_line.strip()]
    has_device = any(output_line.endswith("\tdevice") for output_line in output_lines[1:])
    if not has_device:
        fail("No connected adb device found.")


def ensure_su_available():
    exit_code, _ = run_command(["adb", "shell", "su", "-c", "id"])
    if exit_code != 0:
        fail("Could not execute `su` over adb shell. Ensure the connected device is rooted.")


def ensure_cli_exists(cli_device_path):
    exit_code, _ = run_command(["adb", "shell", "su", "-c", f"test -x {cli_device_path}"])
    if exit_code != 0:
        fail(f"Could not execute `{cli_device_path}` as root. Deploy the worker first with build_and_deploy.py.")


def run_adb_shell(cli_device_path):
    command_segments = ["adb", "shell", "su", "-c", f"{cli_device_path} --ipc-mode"]
    command_display = " ".join(command_segments)
    print(f"> {command_display}")

    process = subprocess.Popen(
        command_segments,
        stdout=None,
        stderr=None,
        text=True,
    )

    process.wait()

    return process.returncode


def main():
    argument_parser = argparse.ArgumentParser(
        description="Run the Android privileged worker in IPC mode via adb + su.",
    )
    argument_parser.add_argument(
        "--cli-path",
        default=DEFAULT_CLI_DEVICE_PATH,
        help="Device path to the squalr-cli binary.",
    )
    parsed_arguments = argument_parser.parse_args()

    ensure_device_connected()
    ensure_su_available()
    ensure_cli_exists(parsed_arguments.cli_path)

    exit_code = run_adb_shell(parsed_arguments.cli_path)
    if exit_code != 0:
        print(f"Privileged shell exited with code {exit_code}.", file=sys.stderr)
        sys.exit(exit_code)


if __name__ == "__main__":
    main()
