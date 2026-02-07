#!/usr/bin/env python3
"""
agentic_loop.py

Runs this command forever in a blocking loop:

  codex exec --yolo "Follow the instructions in AGENTS.MD."

Stop with Ctrl+C.
"""

import subprocess
import sys
import time


CMD = ["codex", "exec", "--yolo", "Follow the instructions in AGENTS.MD."]


def main() -> int:
    iteration = 0
    try:
        while True:
            iteration += 1
            print(f"\n=== Iteration {iteration} ===", flush=True)

            # Blocking call: waits until codex finishes, then loops again.
            completed = subprocess.run(CMD)

            # Optional: small delay to avoid a hot loop if codex exits instantly.
            if completed.returncode != 0:
                print(f"codex exited with code {completed.returncode}", file=sys.stderr, flush=True)
                time.sleep(1)

    except KeyboardInterrupt:
        print("\nStopped (Ctrl+C).", flush=True)
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
