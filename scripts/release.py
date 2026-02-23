#!/usr/bin/env python3
"""Release automation helpers for local and CI workflows."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore

VERSION_PATTERN = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
FIXED_ZIP_TIMESTAMP = (2020, 1, 1, 0, 0, 0)
DESKTOP_CRATES = ["squalr-cli", "squalr-tui", "squalr"]
WINDOWS_INSTALLER_CRATE = "squalr-installer"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Squalr release automation.")
    parser.add_argument(
        "--step",
        choices=["all", "version-bump", "build-package", "release-publish"],
        default="all",
        help="Release step to run.",
    )
    parser.add_argument(
        "--release-type",
        choices=["major", "minor", "patch"],
        help="Version bump type for version-bump/all modes.",
    )
    parser.add_argument(
        "--non-interactive",
        action="store_true",
        help="Disable prompts and fail when required values are missing.",
    )
    parser.add_argument(
        "--no-version-bump",
        action="store_true",
        help="Skip version bump during all mode.",
    )
    parser.add_argument(
        "--build-profile",
        choices=["debug", "release"],
        default="release",
        help="Cargo profile used in build-package mode.",
    )
    parser.add_argument(
        "--artifact-target",
        help="Artifact target label, for example windows-x86_64.",
    )
    parser.add_argument(
        "--dist-dir",
        default="dist/release",
        help="Output directory for packaged artifacts.",
    )
    parser.add_argument(
        "--assets-dir",
        help="Input directory containing release assets for release-publish mode.",
    )
    parser.add_argument("--tag", help="Git tag to publish, for example v1.2.3.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print actions without changing files, building, or publishing.",
    )
    return parser.parse_args()


def run_command(command_segments: list[str], *, cwd: Path, dry_run: bool) -> None:
    command_display = " ".join(command_segments)
    print(f"> {command_display}")
    if dry_run:
        return

    completed_process = subprocess.run(command_segments, cwd=str(cwd), check=False)
    if completed_process.returncode != 0:
        raise RuntimeError(f"Command failed ({completed_process.returncode}): {command_display}")


def load_toml_file(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def get_workspace_member_paths(root_directory: Path) -> list[Path]:
    workspace_toml = load_toml_file(root_directory / "Cargo.toml")
    workspace_members = workspace_toml.get("workspace", {}).get("members", [])
    return [root_directory / workspace_member for workspace_member in workspace_members]


def get_current_version(root_directory: Path) -> str:
    squalr_toml = load_toml_file(root_directory / "squalr" / "Cargo.toml")
    current_version = squalr_toml.get("package", {}).get("version")
    if not isinstance(current_version, str):
        raise RuntimeError("Unable to locate package.version in squalr/Cargo.toml.")
    return current_version


def bump_version(current_version: str, release_type: str) -> str:
    version_match = VERSION_PATTERN.match(current_version)
    if version_match is None:
        raise RuntimeError(f"Unsupported semantic version format: {current_version}")

    major_value, minor_value, patch_value = [int(version_component) for version_component in version_match.groups()]
    if release_type == "major":
        return f"{major_value + 1}.0.0"
    if release_type == "minor":
        return f"{major_value}.{minor_value + 1}.0"
    if release_type == "patch":
        return f"{major_value}.{minor_value}.{patch_value + 1}"

    raise RuntimeError(f"Unsupported release type: {release_type}")


def update_cargo_package_version(cargo_toml_path: Path, new_version: str, *, dry_run: bool) -> bool:
    file_lines = cargo_toml_path.read_text(encoding="utf-8").splitlines(keepends=True)
    in_package_block = False
    updated_version = False

    for line_index, file_line in enumerate(file_lines):
        stripped_line = file_line.strip()

        if stripped_line == "[package]":
            in_package_block = True
            continue

        if stripped_line.startswith("[") and stripped_line != "[package]":
            in_package_block = False

        if in_package_block and stripped_line.startswith("version"):
            file_lines[line_index] = f'version = "{new_version}"\n'
            updated_version = True
            break

    if not updated_version:
        return False

    if not dry_run:
        cargo_toml_path.write_text("".join(file_lines), encoding="utf-8")

    return True


def perform_version_bump(
    root_directory: Path,
    *,
    release_type: str | None,
    non_interactive: bool,
    dry_run: bool,
) -> str:
    current_version = get_current_version(root_directory)

    selected_release_type = release_type
    if selected_release_type is None:
        if non_interactive:
            raise RuntimeError("--release-type is required in --non-interactive mode.")
        selected_release_type = input("Release type (major/minor/patch): ").strip().lower()

    if selected_release_type not in {"major", "minor", "patch"}:
        raise RuntimeError("Release type must be one of: major, minor, patch.")

    new_version = bump_version(current_version, selected_release_type)
    print(f"Current version: {current_version}")
    print(f"New version: {new_version}")

    for workspace_member_path in get_workspace_member_paths(root_directory):
        cargo_toml_path = workspace_member_path / "Cargo.toml"
        if not cargo_toml_path.exists():
            continue
        was_updated = update_cargo_package_version(cargo_toml_path, new_version, dry_run=dry_run)
        if was_updated:
            print(f"Updated version in {cargo_toml_path}")

    return new_version


def resolve_binary_name(crate_name: str) -> str:
    executable_suffix = ".exe" if os.name == "nt" else ""
    return f"{crate_name}{executable_suffix}"


def is_macos_artifact_target(artifact_target: str) -> bool:
    return artifact_target.startswith("macos-")


def ensure_clean_directory(directory_path: Path, *, dry_run: bool) -> None:
    if directory_path.exists():
        if dry_run:
            print(f"Would remove existing directory: {directory_path}")
        else:
            shutil.rmtree(directory_path)

    if dry_run:
        print(f"Would create directory: {directory_path}")
    else:
        directory_path.mkdir(parents=True, exist_ok=True)


def build_desktop_binaries(root_directory: Path, *, build_profile: str, dry_run: bool, include_installer: bool) -> None:
    cargo_build_arguments = ["build", "--locked"]
    if build_profile == "release":
        cargo_build_arguments.append("--release")

    for desktop_crate in DESKTOP_CRATES:
        run_command(
            ["cargo", *cargo_build_arguments, "-p", desktop_crate],
            cwd=root_directory,
            dry_run=dry_run,
        )

    if include_installer:
        run_command(
            ["cargo", *cargo_build_arguments, "-p", WINDOWS_INSTALLER_CRATE],
            cwd=root_directory,
            dry_run=dry_run,
        )


def copy_file_with_parent(source_path: Path, destination_path: Path, *, dry_run: bool) -> None:
    if dry_run:
        print(f"Would copy {source_path} -> {destination_path}")
        return

    destination_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_path, destination_path)


def create_macos_app_bundle(
    *,
    gui_binary_source_path: Path,
    staging_directory: Path,
    version: str,
    dry_run: bool,
) -> Path:
    app_bundle_directory = staging_directory / "squalr.app"
    app_contents_directory = app_bundle_directory / "Contents"
    app_macos_directory = app_contents_directory / "MacOS"
    bundled_gui_binary_path = app_macos_directory / "squalr"
    info_plist_path = app_contents_directory / "Info.plist"

    info_plist_text = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key>
  <string>Squalr</string>
  <key>CFBundleExecutable</key>
  <string>squalr</string>
  <key>CFBundleIdentifier</key>
  <string>com.squalr.desktop</string>
  <key>CFBundleName</key>
  <string>Squalr</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{version}</string>
  <key>CFBundleVersion</key>
  <string>{version}</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
</dict>
</plist>
"""

    if dry_run:
        print(f"Would create macOS app bundle directory: {app_bundle_directory}")
        print(f"Would copy {gui_binary_source_path} -> {bundled_gui_binary_path}")
        print(f"Would write Info.plist: {info_plist_path}")
        return app_bundle_directory

    app_macos_directory.mkdir(parents=True, exist_ok=True)
    shutil.copy2(gui_binary_source_path, bundled_gui_binary_path)
    current_file_mode = bundled_gui_binary_path.stat().st_mode
    bundled_gui_binary_path.chmod(current_file_mode | 0o111)
    info_plist_path.write_text(info_plist_text, encoding="utf-8")
    return app_bundle_directory


def create_deterministic_zip(source_directory: Path, zip_path: Path, *, dry_run: bool) -> None:
    import zipfile

    if dry_run:
        print(f"Would create archive: {zip_path}")
        return

    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zip_file:
        for source_file in sorted(source_directory.rglob("*")):
            if not source_file.is_file():
                continue

            relative_file_path = source_file.relative_to(source_directory).as_posix()
            zip_info = zipfile.ZipInfo(relative_file_path)
            zip_info.date_time = FIXED_ZIP_TIMESTAMP
            zip_info.compress_type = zipfile.ZIP_DEFLATED
            zip_info.create_system = 3
            zip_info.external_attr = (0o755 if os.access(source_file, os.X_OK) else 0o644) << 16
            zip_file.writestr(zip_info, source_file.read_bytes())


def compute_sha256(file_path: Path) -> str:
    sha256_hasher = hashlib.sha256()
    with file_path.open("rb") as file_handle:
        for file_chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            sha256_hasher.update(file_chunk)
    return sha256_hasher.hexdigest()


def write_checksum_file(target_directory: Path, checksum_file_name: str, file_paths: Iterable[Path], *, dry_run: bool) -> Path:
    checksum_file_path = target_directory / checksum_file_name
    checksum_lines: list[str] = []

    if dry_run:
        print(f"Would write checksum file: {checksum_file_path}")
        for file_path in sorted(file_paths):
            print(f"Would hash: {file_path}")
        return checksum_file_path

    for file_path in sorted(file_paths):
        checksum_lines.append(f"{compute_sha256(file_path)}  {file_path.name}\n")

    checksum_file_path.write_text("".join(checksum_lines), encoding="utf-8")

    return checksum_file_path


def validate_required_files(required_paths: list[Path]) -> None:
    missing_paths = [required_path for required_path in required_paths if not required_path.exists()]
    if missing_paths:
        missing_paths_display = "\n".join(str(missing_path) for missing_path in missing_paths)
        raise RuntimeError(f"Required build artifacts are missing:\n{missing_paths_display}")


def package_desktop_artifacts(
    root_directory: Path,
    *,
    version: str,
    artifact_target: str,
    build_profile: str,
    dist_directory: Path,
    dry_run: bool,
) -> None:
    target_profile_directory = root_directory / "target" / build_profile
    include_installer = artifact_target.startswith("windows-")

    build_desktop_binaries(
        root_directory,
        build_profile=build_profile,
        dry_run=dry_run,
        include_installer=include_installer,
    )

    executable_names = [resolve_binary_name(crate_name) for crate_name in DESKTOP_CRATES]
    if include_installer:
        executable_names.append(resolve_binary_name(WINDOWS_INSTALLER_CRATE))

    required_binary_paths = [target_profile_directory / executable_name for executable_name in executable_names]
    if not dry_run:
        validate_required_files(required_binary_paths)

    ensure_clean_directory(dist_directory, dry_run=dry_run)
    staging_directory = dist_directory / "bundle"
    if not dry_run:
        staging_directory.mkdir(parents=True, exist_ok=True)

    bundle_prefix = f"squalr-{version}-{artifact_target}"
    bundle_archive_path = dist_directory / f"{bundle_prefix}.zip"
    manifest_path = dist_directory / f"{bundle_prefix}.manifest.json"
    version_marker_path = dist_directory / f"latest_version-{version}-{artifact_target}.txt"

    packaged_executable_names: list[str] = []
    is_macos_bundle_target = is_macos_artifact_target(artifact_target)

    for executable_name in executable_names:
        source_binary_path = target_profile_directory / executable_name
        if is_macos_bundle_target and executable_name == resolve_binary_name("squalr"):
            create_macos_app_bundle(
                gui_binary_source_path=source_binary_path,
                staging_directory=staging_directory,
                version=version,
                dry_run=dry_run,
            )
            packaged_executable_names.append("squalr.app")
            continue

        destination_binary_path = staging_directory / executable_name
        copy_file_with_parent(source_binary_path, destination_binary_path, dry_run=dry_run)
        packaged_executable_names.append(executable_name)

    if dry_run:
        print(f"Would write version marker: {version_marker_path}")
    else:
        version_marker_path.write_text(f"{version}\n", encoding="utf-8")

    create_deterministic_zip(staging_directory, bundle_archive_path, dry_run=dry_run)

    artifact_manifest = {
        "version": version,
        "artifact_target": artifact_target,
        "build_profile": build_profile,
        "artifacts": [bundle_archive_path.name, version_marker_path.name],
        "executables": packaged_executable_names,
    }
    if dry_run:
        print(f"Would write manifest: {manifest_path}")
    else:
        manifest_path.write_text(json.dumps(artifact_manifest, indent=2) + "\n", encoding="utf-8")

    checksum_candidates = [bundle_archive_path, version_marker_path, manifest_path]
    checksum_file_path = write_checksum_file(
        dist_directory,
        checksum_file_name=f"SHA256SUMS-{artifact_target}.txt",
        file_paths=checksum_candidates,
        dry_run=dry_run,
    )

    expected_dist_files = checksum_candidates + [checksum_file_path]
    if not dry_run:
        validate_required_files(expected_dist_files)
        shutil.rmtree(staging_directory)

    print(f"Packaged artifacts in {dist_directory}")


def discover_release_assets(assets_directory: Path) -> list[Path]:
    release_assets: list[Path] = []
    for file_path in sorted(assets_directory.rglob("*")):
        if not file_path.is_file():
            continue

        file_name = file_path.name
        if file_name.startswith("latest_version-") and file_name.endswith(".txt"):
            release_assets.append(file_path)
            continue
        if file_name.startswith("SHA256SUMS") and file_name.endswith(".txt"):
            release_assets.append(file_path)
            continue
        if file_name.endswith(".manifest.json"):
            release_assets.append(file_path)
            continue
        if file_name.endswith((".zip", ".apk", ".exe", ".dmg", ".gz", ".tgz")):
            release_assets.append(file_path)

    return release_assets


def run_gh_command(command_segments: list[str], *, dry_run: bool) -> subprocess.CompletedProcess[bytes] | None:
    command_display = " ".join(command_segments)
    print(f"> {command_display}")
    if dry_run:
        return None

    completed_process = subprocess.run(command_segments, check=False, capture_output=True)
    if completed_process.returncode != 0:
        stderr_text = completed_process.stderr.decode("utf-8", errors="ignore")
        stdout_text = completed_process.stdout.decode("utf-8", errors="ignore")
        raise RuntimeError(
            f"Command failed ({completed_process.returncode}): {command_display}\n"
            f"stdout:\n{stdout_text}\n"
            f"stderr:\n{stderr_text}"
        )
    return completed_process


def publish_release(
    *,
    tag_name: str,
    assets_directory: Path,
    dry_run: bool,
) -> None:
    if not re.match(r"^v\d+\.\d+\.\d+$", tag_name):
        raise RuntimeError("Release tag must match v<major>.<minor>.<patch>.")

    if not assets_directory.exists():
        raise RuntimeError(f"Assets directory does not exist: {assets_directory}")

    release_assets = discover_release_assets(assets_directory)
    if not release_assets:
        raise RuntimeError(f"No release assets discovered in {assets_directory}")

    combined_checksums_path = assets_directory / "SHA256SUMS.txt"
    write_checksum_file(
        assets_directory,
        checksum_file_name=combined_checksums_path.name,
        file_paths=release_assets,
        dry_run=dry_run,
    )

    if not dry_run:
        release_assets = discover_release_assets(assets_directory)

    if not dry_run:
        release_exists_process = subprocess.run(["gh", "release", "view", tag_name], check=False, capture_output=True)
        release_exists = release_exists_process.returncode == 0
    else:
        release_exists = False

    if release_exists:
        run_gh_command(["gh", "release", "edit", tag_name, "--draft=true"], dry_run=dry_run)
    else:
        release_title = f"Squalr {tag_name}"
        run_gh_command(
            [
                "gh",
                "release",
                "create",
                tag_name,
                "--title",
                release_title,
                "--notes",
                "Automated release draft generated by CI.",
                "--draft",
            ],
            dry_run=dry_run,
        )

    upload_command = ["gh", "release", "upload", tag_name, "--clobber", *[str(asset_path) for asset_path in release_assets]]
    run_gh_command(upload_command, dry_run=dry_run)


def main() -> None:
    parsed_arguments = parse_args()
    root_directory = Path(__file__).resolve().parent.parent

    selected_steps = (
        ["version-bump", "build-package", "release-publish"]
        if parsed_arguments.step == "all"
        else [parsed_arguments.step]
    )

    current_version = get_current_version(root_directory)
    target_version = current_version

    if "version-bump" in selected_steps and not parsed_arguments.no_version_bump:
        target_version = perform_version_bump(
            root_directory,
            release_type=parsed_arguments.release_type,
            non_interactive=parsed_arguments.non_interactive,
            dry_run=parsed_arguments.dry_run,
        )

    if "build-package" in selected_steps:
        if not parsed_arguments.artifact_target:
            raise RuntimeError("--artifact-target is required for build-package mode.")
        if parsed_arguments.tag:
            expected_tag_version = parsed_arguments.tag.removeprefix("v")
            if expected_tag_version != target_version:
                raise RuntimeError(
                    f"Tag version ({expected_tag_version}) does not match workspace version ({target_version})."
                )

        dist_directory = (root_directory / parsed_arguments.dist_dir).resolve()
        package_desktop_artifacts(
            root_directory,
            version=target_version,
            artifact_target=parsed_arguments.artifact_target,
            build_profile=parsed_arguments.build_profile,
            dist_directory=dist_directory,
            dry_run=parsed_arguments.dry_run,
        )

    if "release-publish" in selected_steps:
        if not parsed_arguments.tag:
            raise RuntimeError("--tag is required for release-publish mode.")
        if not parsed_arguments.assets_dir:
            raise RuntimeError("--assets-dir is required for release-publish mode.")

        assets_directory = (root_directory / parsed_arguments.assets_dir).resolve()
        publish_release(tag_name=parsed_arguments.tag, assets_directory=assets_directory, dry_run=parsed_arguments.dry_run)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Error: {error}")
        sys.exit(1)
