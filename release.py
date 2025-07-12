#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
from pathlib import Path
import toml
import semver

def get_workspace_members(root_dir):
    """Read workspace members from root Cargo.toml."""
    cargo_toml = toml.load(os.path.join(root_dir, "Cargo.toml"))
    return cargo_toml.get("workspace", {}).get("members", [])

def get_current_version(root_dir):
    """Get current version from olorin/Cargo.toml."""
    olorin_cargo = toml.load(os.path.join(root_dir, "olorin", "Cargo.toml"))
    return olorin_cargo.get("package", {}).get("version")

def increment_version(current_version, release_type):
    """Increment version based on release type."""
    ver = semver.VersionInfo.parse(current_version)
    if release_type == "major":
        return str(ver.bump_major())
    elif release_type == "minor":
        return str(ver.bump_minor())
    elif release_type == "patch":
        return str(ver.bump_patch())
    else:
        raise ValueError("Invalid release type")

# This is a bit brittle, manually navigating the .toml format.
# This could be more robust using cargo toml dumping, but then the format
# of the file gets screwed up. Easiest just to regex it for now, with some manual sanity checks.
def update_cargo_toml_version(cargo_path, new_version):
    try:
        with open(cargo_path, 'r') as f:
            lines = f.readlines()
        
        in_package_section = False
        for i, line in enumerate(lines):
            line = line.strip()
            if line == '[package]':
                in_package_section = True
            elif line.startswith('['):
                in_package_section = False
            elif in_package_section and line.startswith('version ='):
                lines[i] = f'version = "{new_version}"\n'
        
        with open(cargo_path, 'w') as f:
            f.writelines(lines)
    except Exception as e:
        print(f"Warning: Could not update version in {cargo_path}: {e}")

def run_cargo_command(cwd, command):
    """Run a cargo command in the specified directory."""
    try:
        subprocess.run(["cargo"] + command, cwd=cwd, check=True)
    except subprocess.CalledProcessError as e:
        print(f"Error running cargo command in {cwd}: {e}")
        sys.exit(1)

def ensure_dist_directory(root_dir):
    """Ensure dist directory exists and is empty."""
    dist_dir = os.path.join(root_dir, "dist")
    if os.path.exists(dist_dir):
        shutil.rmtree(dist_dir)
    os.makedirs(dist_dir)
    return dist_dir

def copy_binaries_and_resources(root_dir, dist_dir):
    """Copy binaries and resource directories to dist directory."""
    # Assuming release builds
    target_dir = os.path.join(root_dir, "target", "release")
    
    # Create olorin directory in dist for the full package
    olorin_dist_dir = os.path.join(dist_dir, "olorin")
    os.makedirs(olorin_dist_dir, exist_ok=True)
    
    # Copy olorin binary
    olorin_binary = "olorin.exe" if os.name == 'nt' else "olorin"
    if os.path.exists(os.path.join(target_dir, olorin_binary)):
        shutil.copy2(
            os.path.join(target_dir, olorin_binary),
            os.path.join(olorin_dist_dir, olorin_binary)
        )
    
    # Copy olorin-installer binary and latest_version to dist root
    installer_binary = "olorin-installer.exe" if os.name == 'nt' else "olorin-installer"
    if os.path.exists(os.path.join(target_dir, installer_binary)):
        shutil.copy2(
            os.path.join(target_dir, installer_binary),
            os.path.join(dist_dir, installer_binary)
        )
    
    latest_version_file = os.path.join(target_dir, "latest_version")
    if os.path.exists(latest_version_file):
        shutil.copy2(
            latest_version_file,
            os.path.join(dist_dir, "latest_version")
        )
    else:
        print(f"Warning: latest_version file not found in build output at {latest_version_file}")
    
    # Copy resource directories from build output
    resource_dirs = ['third-party', 'gifs', 'audio']
    for dir_name in resource_dirs:
        src_dir = os.path.join(target_dir, dir_name)
        if os.path.exists(src_dir):
            dst_dir = os.path.join(olorin_dist_dir, dir_name)
            print(f"Copying {dir_name}...")
            shutil.copytree(src_dir, dst_dir, dirs_exist_ok=True)
        else:
            print(f"Warning: Required directory {dir_name} not found in build output at {src_dir}")
    
    # Create zip archive
    print("Creating olorin.zip archive...")
    shutil.make_archive(
        os.path.join(dist_dir, "olorin"),  # base name
        'zip',                           # format
        olorin_dist_dir                    # root_dir
    )
    
    # Delete the olorin directory after creating the zip
    print("Cleaning up olorin directory...")
    shutil.rmtree(olorin_dist_dir)

def main():
    root_dir = os.getcwd()
    
    # Get current version
    current_version = get_current_version(root_dir)
    print(f"Current version is {current_version}")
    
    # Get release type
    while True:
        release_type = input("Please specify a release type: 'major', 'minor', or 'patch': ").lower()
        if release_type in ["major", "minor", "patch"]:
            break
        print("Please enter 'major', 'minor', or 'patch'")
    
    # Calculate new version
    new_version = increment_version(current_version, release_type)
    print(f"New version will be: {new_version}")
    
    # Update versions in all workspace members
    members = get_workspace_members(root_dir)
    for member in members:
        cargo_path = os.path.join(root_dir, member, "Cargo.toml")
        if os.path.exists(cargo_path):
            update_cargo_toml_version(cargo_path, new_version)
            print(f"Updated version in {member}/Cargo.toml")
    
    # Ensure dist directory exists and is empty
    dist_dir = ensure_dist_directory(root_dir)
    
    # Build olorin (Win64)
    print("Building olorin (Win64)...")
    run_cargo_command(os.path.join(root_dir, "olorin"), ["build", "--release"])
    
    # Build olorin-installer
    print("Building olorin-installer...")
    run_cargo_command(os.path.join(root_dir, "olorin-installer"), ["build", "--release"])
    
    # Copy binaries and resources to dist directory
    print("Copying binaries and resources to dist directory...")
    copy_binaries_and_resources(root_dir, dist_dir)
    
    print("""
Build complete! 
Location of files:
- dist/olorin-installer.exe - Standalone installer
- dist/olorin.zip - Complete package including olorin executable and resources
- dist/latest_version - Version information for the installer
    """)
    print(f"New version is: {new_version}")

if __name__ == "__main__":
    main()
