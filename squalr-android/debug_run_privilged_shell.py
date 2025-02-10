import subprocess

def run_adb_command():
    command = 'adb shell su -c "/data/data/rust.squalr_android/files/squalr-cli --ipc-mode"'
    
    try:
        # Use Popen for better interaction
        process = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

        # Read output line by line (prevents hanging)
        while True:
            output = process.stdout.readline()
            if output == "" and process.poll() is not None:
                break
            if output:
                print(output.strip())

        # Capture and print any errors
        stderr = process.stderr.read()
        if stderr:
            print("Error:\n", stderr.strip())

    except Exception as e:
        print(f"Exception occurred: {e}")

if __name__ == "__main__":
    run_adb_command()
