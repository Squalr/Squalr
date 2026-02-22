import argparse
import subprocess
import sys


DEFAULT_CLI_DEVICE_PATH = "/data/local/tmp/squalr-cli"


def run_adb_shell(cli_device_path):
    command_segments = ["adb", "shell", "su", "-c", f"{cli_device_path} --ipc-mode"]
    command_display = " ".join(command_segments)
    print(f"> {command_display}")

    process = subprocess.Popen(
        command_segments,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    if process.stdout is not None:
        for output_line in process.stdout:
            print(output_line, end="")

    process.wait()

    if process.stderr is not None:
        error_output = process.stderr.read()
        if error_output:
            print(error_output, file=sys.stderr, end="")

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

    exit_code = run_adb_shell(parsed_arguments.cli_path)
    if exit_code != 0:
        print(f"Privileged shell exited with code {exit_code}.", file=sys.stderr)
        sys.exit(exit_code)


if __name__ == "__main__":
    main()
