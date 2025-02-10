import subprocess
import os

def run_command(command):
    """Runs a shell command and streams the output in real-time."""
    process = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    for line in process.stdout:
        print(line, end='')
    process.wait()
    if process.returncode != 0:
        print(f"Command failed: {command}")
        exit(process.returncode)

# Get the script directory and parent directory
script_dir = os.path.dirname(os.path.abspath(__file__))
parent_dir = os.path.abspath(os.path.join(script_dir, os.pardir))

# Prompt user for release mode
display_mode = input("Build in release mode? (y/n [default]): ").strip().lower()
release_flag = "--release" if display_mode == "y" else ""

# Run cargo ndk build in chosen mode in parent directory
os.chdir(parent_dir)
run_command(f"cargo ndk --target aarch64-linux-android build --bin squalr-cli {release_flag} -v")

# Run cargo apk build in chosen mode in script directory
os.chdir(script_dir)
run_command(f"cargo apk build --target aarch64-linux-android {release_flag} --lib")

# Install APK using adb from parent directory
apk_path = os.path.join(parent_dir, "target", "release" if release_flag else "debug", "apk", "squalr-android.apk")
run_command(f"adb install {apk_path}")
