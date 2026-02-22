import argparse
import subprocess
import sys


DEFAULT_CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"
SU_INVOCATION_ATTEMPTS = [
    ("su -c", ["su", "-c"]),
    ("su 0 sh -c", ["su", "0", "sh", "-c"]),
    ("su root sh -c", ["su", "root", "sh", "-c"]),
]


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


def run_su_command_with_fallback(shell_command, action_label):
    for su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
        exit_code, _ = run_command(["adb", "shell", *su_invocation_prefix, shell_command])
        if exit_code == 0:
            print(f"Succeeded: {action_label} via {su_invocation_label}")
            return

    fail(
        f"Failed to {action_label} with all su invocation attempts: "
        + ", ".join(invocation_label for invocation_label, _ in SU_INVOCATION_ATTEMPTS)
    )


def ensure_device_connected():
    exit_code, output_text = run_command(["adb", "devices"])
    if exit_code != 0:
        fail("Failed to query adb devices.")

    output_lines = [output_line.strip() for output_line in output_text.splitlines() if output_line.strip()]
    has_device = any(output_line.endswith("\tdevice") for output_line in output_lines[1:])
    if not has_device:
        fail("No connected adb device found.")


def ensure_su_available():
    run_su_command_with_fallback("id", "execute su over adb shell")


def ensure_cli_exists(cli_device_path):
    run_su_command_with_fallback(
        f"test -x {cli_device_path}",
        f"validate executable worker path {cli_device_path}",
    )


def run_adb_shell(cli_device_path):
    for su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
        command_segments = ["adb", "shell", *su_invocation_prefix, f"{cli_device_path} --ipc-mode"]
        command_display = " ".join(command_segments)
        print(f"> {command_display}")

        process = subprocess.Popen(
            command_segments,
            stdout=None,
            stderr=None,
            text=True,
        )
        process.wait()

        if process.returncode == 0:
            print(f"Privileged worker shell exited successfully via {su_invocation_label}.")
            return 0

        print(f"Invocation failed via {su_invocation_label} with exit code {process.returncode}.")

    return 1


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
