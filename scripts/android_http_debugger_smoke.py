#!/usr/bin/env python3

import argparse
import http.client
import json
import pathlib
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request


WORKSPACE_ROOT = pathlib.Path(__file__).resolve().parents[1]
ANDROID_TARGET = "aarch64-linux-android"
HTTP_BIND_ADDRESS = "127.0.0.1:49321"
HTTP_BASE_URL = f"http://{HTTP_BIND_ADDRESS}"
REMOTE_CLI_PATH = "/data/local/tmp/squalr-cli"
REMOTE_TARGET_PATH = "/data/local/tmp/squalr-debugger-watch-target"
REMOTE_HTTP_LOG_PATH = "/data/local/tmp/squalr-http-ipc.log"
REMOTE_TARGET_LOG_PATH = "/data/local/tmp/squalr-debugger-watch-target.log"
READY_LINE_PATTERN = re.compile(r"READY pid=(?P<process_id>\d+) address=(?P<address>0x[0-9a-fA-F]+) size=(?P<size>\d+)")
HEARTBEAT_LINE_PATTERN = re.compile(r"HEARTBEAT sequence=(?P<sequence>\d+) value=(?P<value>\d+)")


class SmokeFailure(RuntimeError):
    pass


def run_host_command(command_arguments, *, timeout_seconds=120):
    print(f"+ {' '.join(str(command_argument) for command_argument in command_arguments)}", flush=True)
    completed_process = subprocess.run(
        [str(command_argument) for command_argument in command_arguments],
        cwd=WORKSPACE_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout_seconds,
    )

    if completed_process.returncode != 0:
        raise SmokeFailure(
            f"Command failed with exit code {completed_process.returncode}: {' '.join(str(argument) for argument in command_arguments)}\n"
            f"{completed_process.stdout}"
        )

    if completed_process.stdout.strip():
        print(completed_process.stdout, end="" if completed_process.stdout.endswith("\n") else "\n")

    return completed_process.stdout


def run_adb_shell(command_text, *, timeout_seconds=30, allow_failure=False):
    completed_process = subprocess.run(
        ["adb", "shell", "su", "0", "sh", "-c", command_text],
        cwd=WORKSPACE_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout_seconds,
    )

    if completed_process.returncode != 0 and not allow_failure:
        raise SmokeFailure(f"adb shell command failed with exit code {completed_process.returncode}: {command_text}\n{completed_process.stdout}")

    return completed_process.stdout


def build_android_artifacts(skip_build):
    if skip_build:
        return

    run_host_command(
        ["cargo", "ndk", "--target", ANDROID_TARGET, "build", "-p", "squalr-cli", "--locked", "--release"],
        timeout_seconds=300,
    )
    run_host_command(
        [
            "cargo",
            "ndk",
            "--target",
            ANDROID_TARGET,
            "build",
            "-p",
            "squalr-plugin-debuggers-native",
            "--example",
            "debugger_watch_target",
            "--locked",
            "--release",
        ],
        timeout_seconds=300,
    )


def push_android_artifacts():
    cli_artifact_path = WORKSPACE_ROOT / "target" / ANDROID_TARGET / "release" / "squalr-cli"
    target_artifact_path = WORKSPACE_ROOT / "target" / ANDROID_TARGET / "release" / "examples" / "debugger_watch_target"

    if not cli_artifact_path.exists():
        raise SmokeFailure(f"Missing CLI artifact: {cli_artifact_path}")
    if not target_artifact_path.exists():
        raise SmokeFailure(f"Missing watch-target artifact: {target_artifact_path}")

    run_host_command(["adb", "push", cli_artifact_path, REMOTE_CLI_PATH], timeout_seconds=60)
    run_host_command(["adb", "push", target_artifact_path, REMOTE_TARGET_PATH], timeout_seconds=60)
    run_adb_shell(f"chmod +x {REMOTE_CLI_PATH} {REMOTE_TARGET_PATH}")


def reset_remote_processes():
    run_adb_shell(f"pkill -f {REMOTE_CLI_PATH} || true", allow_failure=True)
    run_adb_shell(f"pkill -f {REMOTE_TARGET_PATH} || true", allow_failure=True)
    run_adb_shell(f"rm -f {REMOTE_HTTP_LOG_PATH} {REMOTE_TARGET_LOG_PATH}", allow_failure=True)


def launch_http_ipc_server():
    run_adb_shell(f"{REMOTE_CLI_PATH} --http-ipc {HTTP_BIND_ADDRESS} >{REMOTE_HTTP_LOG_PATH} 2>&1 &")
    run_host_command(["adb", "forward", "tcp:49321", "tcp:49321"], timeout_seconds=30)

    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            health_response = http_get_json("/health", timeout_seconds=2)
            if health_response.get("success") is True:
                print("HTTP IPC health check passed.", flush=True)
                return
        except (SmokeFailure, TimeoutError, http.client.RemoteDisconnected, urllib.error.URLError):
            time.sleep(0.25)

    http_log = run_adb_shell(f"cat {REMOTE_HTTP_LOG_PATH} || true", allow_failure=True)
    raise SmokeFailure(f"Timed out waiting for HTTP IPC health response.\nHTTP log:\n{http_log}")


def launch_watch_target():
    run_adb_shell(f"{REMOTE_TARGET_PATH} >{REMOTE_TARGET_LOG_PATH} 2>&1 &")

    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        target_log = run_adb_shell(f"cat {REMOTE_TARGET_LOG_PATH} || true", allow_failure=True)
        ready_match = READY_LINE_PATTERN.search(target_log)
        if ready_match:
            process_id = int(ready_match.group("process_id"))
            target_address = int(ready_match.group("address"), 16)
            target_size = int(ready_match.group("size"))
            print(f"Watch target ready: pid={process_id}, address={target_address:#x}, size={target_size}.", flush=True)
            return process_id, target_address, target_size

        time.sleep(0.25)

    raise SmokeFailure(f"Timed out waiting for watch target readiness.\nTarget log:\n{target_log}")


def http_get_json(path, *, timeout_seconds=10):
    request = urllib.request.Request(f"{HTTP_BASE_URL}{path}", method="GET")
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        return json.loads(response.read().decode("utf-8"))


def http_post_command(command_payload, *, timeout_seconds=30):
    request_body = json.dumps(command_payload).encode("utf-8")
    request = urllib.request.Request(
        f"{HTTP_BASE_URL}/command",
        data=request_body,
        method="POST",
        headers={"Content-Type": "application/json"},
    )

    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            command_response = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        error_body = error.read().decode("utf-8", errors="replace")
        raise SmokeFailure(f"HTTP command failed with status {error.code}: {error_body}") from error

    if command_response.get("success") is not True:
        raise SmokeFailure(f"HTTP command returned failure envelope: {json.dumps(command_response, indent=2)}")

    return command_response["result"]["privileged_command_response"]


def open_process(process_id):
    process_response = http_post_command(
        {
            "Process": {
                "Open": {
                    "process_open_request": {
                        "process_id": process_id,
                        "search_name": None,
                        "match_case": False,
                    }
                }
            }
        }
    )
    opened_process_info = process_response["Process"]["Open"]["process_open_response"]["opened_process_info"]
    if opened_process_info is None:
        raise SmokeFailure(f"Process.Open returned no opened_process_info for pid {process_id}.")
    print(f"Opened process through HTTP IPC: {opened_process_info.get('name', '<unknown>')}.", flush=True)


def start_trace(target_address, target_size, access_label):
    debugger_response = http_post_command(
        {
            "Debugger": {
                "TraceStart": {
                    "debugger_trace_start_request": {
                        "address": target_address,
                        "size_in_bytes": target_size,
                        "access": access_label,
                        "label": f"android-http-smoke-{access_label.lower()}",
                    }
                }
            }
        },
        timeout_seconds=45,
    )
    trace_start_response = debugger_response["Debugger"]["TraceStart"]["debugger_trace_start_response"]
    require_debugger_status(trace_start_response["status"], f"TraceStart {access_label}")
    trace_session = trace_start_response["trace_session"]
    if trace_session is None:
        raise SmokeFailure(f"TraceStart {access_label} succeeded without a trace_session.")

    trace_session_id = trace_session["trace_session_id"]
    print(f"Started {access_label} trace session {trace_session_id}.", flush=True)
    return trace_session_id


def stop_trace(trace_session_id):
    debugger_response = http_post_command(
        {
            "Debugger": {
                "TraceStop": {
                    "debugger_trace_stop_request": {
                        "trace_session_id": trace_session_id,
                    }
                }
            }
        },
        timeout_seconds=45,
    )
    trace_stop_response = debugger_response["Debugger"]["TraceStop"]["debugger_trace_stop_response"]
    require_debugger_status(trace_stop_response["status"], f"TraceStop {trace_session_id}")
    print(f"Stopped trace session {trace_session_id}.", flush=True)


def detach_debugger():
    debugger_response = http_post_command({"Debugger": {"Detach": {"debugger_detach_request": {}}}}, timeout_seconds=45)
    detach_response = debugger_response["Debugger"]["Detach"]["debugger_detach_response"]
    require_debugger_status(detach_response["status"], "Debugger.Detach")
    print("Detached debugger.", flush=True)


def list_trace_records(trace_session_id):
    debugger_response = http_post_command({"Debugger": {"TraceList": {"debugger_trace_list_request": {}}}}, timeout_seconds=30)
    trace_list_response = debugger_response["Debugger"]["TraceList"]["debugger_trace_list_response"]
    require_debugger_status(trace_list_response["status"], "TraceList")
    return [
        instruction_record
        for instruction_record in trace_list_response["instruction_records"]
        if instruction_record.get("trace_session_id") == trace_session_id
    ]


def require_debugger_status(status, operation_name):
    if status.get("success") is not True:
        raise SmokeFailure(f"{operation_name} failed: {status.get('message') or 'no diagnostic message'}")


def wait_for_trace_event(after_event_id, access_label, *, timeout_seconds=15):
    deadline = time.monotonic() + timeout_seconds
    latest_event_id = after_event_id

    while time.monotonic() < deadline:
        events_response = http_get_json(f"/events?after={latest_event_id}", timeout_seconds=5)
        latest_event_id = max(latest_event_id, int(events_response.get("latest_event_id", latest_event_id)))

        for event_envelope in events_response.get("events", []):
            latest_event_id = max(latest_event_id, int(event_envelope["event_id"]))
            trace_event = extract_trace_event(event_envelope)
            if trace_event is not None:
                instruction_text = trace_event.get("instruction_text")
                instruction_address = trace_event.get("instruction_address")
                instruction_bytes = trace_event.get("instruction_bytes") or []
                if instruction_address is None:
                    raise SmokeFailure(f"{access_label} trace event did not include an instruction address.")
                if not instruction_bytes:
                    raise SmokeFailure(f"{access_label} trace event did not include instruction bytes.")
                print(
                    f"{access_label} trace event: instruction_address={format_optional_address(instruction_address)}, "
                    f"bytes={len(instruction_bytes)}, instruction={instruction_text!r}.",
                    flush=True,
                )
                return latest_event_id, trace_event

        time.sleep(0.1)

    raise SmokeFailure(f"Timed out waiting for {access_label} trace event after event id {after_event_id}.")


def extract_trace_event(event_envelope):
    engine_event = event_envelope.get("event", {}).get("engine_event")
    if not isinstance(engine_event, dict):
        return None

    debugger_event = engine_event.get("Debugger")
    if not isinstance(debugger_event, dict):
        return None

    trace_recorded = debugger_event.get("TraceRecorded")
    if not isinstance(trace_recorded, dict):
        return None

    recorded_event = trace_recorded.get("debugger_trace_recorded_event")
    if not isinstance(recorded_event, dict):
        return None

    return recorded_event.get("trace_event")


def get_latest_event_id():
    events_response = http_get_json("/events?after=0", timeout_seconds=5)
    return int(events_response.get("latest_event_id", 0))


def read_latest_heartbeat():
    target_log = run_adb_shell(f"tail -n 20 {REMOTE_TARGET_LOG_PATH} || true", allow_failure=True)
    latest_heartbeat = None

    for log_line in target_log.splitlines():
        heartbeat_match = HEARTBEAT_LINE_PATTERN.search(log_line)
        if heartbeat_match:
            latest_heartbeat = (
                int(heartbeat_match.group("sequence")),
                int(heartbeat_match.group("value")),
            )

    return latest_heartbeat


def require_heartbeat_progress(process_id, previous_heartbeat, operation_name, *, timeout_seconds=5):
    deadline = time.monotonic() + timeout_seconds

    while time.monotonic() < deadline:
        current_heartbeat = read_latest_heartbeat()
        if current_heartbeat is not None and previous_heartbeat is not None and current_heartbeat[0] > previous_heartbeat[0]:
            print(f"Target heartbeat progressed after {operation_name}: {previous_heartbeat[0]} -> {current_heartbeat[0]}.", flush=True)
            return current_heartbeat

        time.sleep(0.25)

    process_status = run_adb_shell(f"cat /proc/{process_id}/status 2>/dev/null || true", allow_failure=True)
    raise SmokeFailure(f"Watch target heartbeat did not progress after {operation_name}. Last status probe: {process_status}")


def require_no_persistent_tracing_stop(process_id, operation_name):
    task_states = run_adb_shell(
        f"for task_status in /proc/{process_id}/task/*/status; do sed -n 's/^State:/State:/p' \"$task_status\"; done",
        allow_failure=True,
    )
    tracing_stop_lines = [task_state for task_state in task_states.splitlines() if "tracing stop" in task_state.lower()]

    if tracing_stop_lines:
        raise SmokeFailure(f"Target has thread(s) stuck in tracing stop after {operation_name}:\n{task_states}")


def format_optional_address(address):
    if address is None:
        return "<none>"

    return f"{int(address):#x}"


def run_trace_flow(process_id, target_address, target_size, access_label, after_event_id):
    heartbeat_before_trace = read_latest_heartbeat()
    trace_session_id = start_trace(target_address, target_size, access_label)
    latest_event_id, trace_event = wait_for_trace_event(after_event_id, access_label)
    trace_records = list_trace_records(trace_session_id)

    if not trace_records:
        raise SmokeFailure(f"TraceList returned no instruction records for session {trace_session_id}.")

    stop_trace(trace_session_id)
    detach_debugger()
    heartbeat_after_trace = require_heartbeat_progress(process_id, heartbeat_before_trace, f"{access_label} trace")
    require_no_persistent_tracing_stop(process_id, f"{access_label} trace")
    print(f"{access_label} trace captured {len(trace_records)} instruction record(s).", flush=True)

    return latest_event_id, heartbeat_after_trace, trace_event


def parse_arguments():
    argument_parser = argparse.ArgumentParser(description="Run Android HTTP IPC debugger smoke validation.")
    argument_parser.add_argument("--skip-build", action="store_true", help="Reuse existing release Android artifacts.")
    return argument_parser.parse_args()


def main():
    arguments = parse_arguments()

    try:
        build_android_artifacts(arguments.skip_build)
        reset_remote_processes()
        push_android_artifacts()
        launch_http_ipc_server()
        process_id, target_address, target_size = launch_watch_target()
        open_process(process_id)

        latest_event_id = get_latest_event_id()
        latest_heartbeat = read_latest_heartbeat()
        if latest_heartbeat is None:
            raise SmokeFailure("Watch target did not emit an initial heartbeat.")

        for access_label in ["Write", "ReadWrite", "Read"]:
            latest_event_id, latest_heartbeat, _trace_event = run_trace_flow(
                process_id,
                target_address,
                target_size,
                access_label,
                latest_event_id,
            )

        print("Android HTTP IPC debugger smoke passed.", flush=True)
    finally:
        run_adb_shell(f"pkill -f {REMOTE_TARGET_PATH} || true", allow_failure=True)
        run_adb_shell(f"pkill -f {REMOTE_CLI_PATH} || true", allow_failure=True)


if __name__ == "__main__":
    try:
        main()
    except SmokeFailure as smoke_failure:
        print(f"ERROR: {smoke_failure}", file=sys.stderr)
        sys.exit(1)
