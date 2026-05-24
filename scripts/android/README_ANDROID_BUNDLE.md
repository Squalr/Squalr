# Squalr Android Artifact Bundle

This folder contains Android release artifacts for manual installation with `adb`.

## Files
- `squalr.apk`: Android GUI application package.
- `squalr-cli`: Privileged worker binary for rooted devices.
- `install-android.sh`: Unix install helper script.
- `install-android.ps1`: Windows PowerShell install helper script.

## Quick Install
On Unix:
```sh
chmod +x ./install-android.sh
./install-android.sh
```

On Windows PowerShell:
```powershell
.\install-android.ps1
```

## Notes
- A rooted Android device is required for privileged worker deployment.
- `adb` must be installed and available in `PATH`.
- The scripts push `squalr-cli` to `/data/local/tmp/squalr-cli` and attempt to run `chmod +x` through `su`.
