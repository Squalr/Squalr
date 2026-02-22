import argparse
import subprocess
import sys
import time


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
            return su_invocation_label

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


def find_worker_pid():
    for su_invocation_label, su_invocation_prefix in SU_INVOCATION_ATTEMPTS:
        exit_code, output_text = run_command(["adb", "shell", *su_invocation_prefix, "pidof squalr-cli"])
        worker_process_identifier = output_text.strip()
        if exit_code == 0 and worker_process_identifier:
            print(f"Detected running worker pid(s) via {su_invocation_label}: {worker_process_identifier}")
            return worker_process_identifier, su_invocation_label

    return None, None


def kill_existing_worker_if_present():
    existing_worker_identifier, existing_worker_invocation = find_worker_pid()
    if existing_worker_identifier is None:
        print("No existing squalr-cli worker process detected before launch.")
        return

    run_su_command_with_fallback("pkill -f squalr-cli", "terminate pre-existing squalr-cli worker")
    print(
        "Terminated pre-existing worker process before diagnostics "
        f"(detected via {existing_worker_invocation})."
    )


def run_adb_shell_with_pid_polling(cli_device_path, launch_poll_seconds):
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
        launch_deadline = time.time() + launch_poll_seconds

        while time.time() < launch_deadline:
            worker_process_identifier, worker_poll_invocation = find_worker_pid()
            if worker_process_identifier:
                print(
                    "Succeeded: launch privileged worker process "
                    f"via {su_invocation_label}; pid poll succeeded via {worker_poll_invocation}."
                )
                process.terminate()
                process.wait(timeout=5)
                return 0

            process_exit_code = process.poll()
            if process_exit_code is not None:
                print(f"Invocation failed via {su_invocation_label} with exit code {process_exit_code}.")
                break

            time.sleep(1)

        if process.poll() is None:
            print(
                f"No worker pid detected within {launch_poll_seconds} second(s) via {su_invocation_label}; "
                "terminating this launch attempt."
            )
            process.terminate()
            process.wait(timeout=5)

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
    argument_parser.add_argument(
        "--launch-poll-seconds",
        type=int,
        default=10,
        help="Seconds to wait for worker pid detection after each launch attempt.",
    )
    parsed_arguments = argument_parser.parse_args()

    ensure_device_connected()
    ensure_su_available()
    ensure_cli_exists(parsed_arguments.cli_path)
    kill_existing_worker_if_present()

    exit_code = run_adb_shell_with_pid_polling(parsed_arguments.cli_path, parsed_arguments.launch_poll_seconds)
    if exit_code != 0:
        print(f"Privileged shell exited with code {exit_code}.", file=sys.stderr)
        sys.exit(exit_code)


if __name__ == "__main__":
    main()
