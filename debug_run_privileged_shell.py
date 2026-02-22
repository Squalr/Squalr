import subprocess
import sys
from pathlib import Path


def main() -> int:
    workspace_directory = Path(__file__).resolve().parent
    target_script_path = workspace_directory / "squalr-android" / "debug_run_privileged_shell.py"
    command_segments = [sys.executable, str(target_script_path), *sys.argv[1:]]
    return subprocess.call(command_segments, cwd=workspace_directory)


if __name__ == "__main__":
    sys.exit(main())
