#!/usr/bin/env python3
"""
Vendor ts-pack-core crate into PHP package for extension distribution.

This script:
1. Reads workspace.dependencies from root Cargo.toml
2. Copies ts-pack-core to crates/ts-pack-php/vendor/
3. Replaces workspace = true with explicit versions
4. Generates vendor/Cargo.toml with proper workspace setup
"""

import os
import re
import shutil
import sys
from collections.abc import Callable
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore[no-redef]


def get_repo_root() -> Path:
    """Get repository root directory."""
    repo_root_env = os.environ.get("PROJECT_ROOT")
    if repo_root_env:
        return Path(repo_root_env)

    script_dir = Path(__file__).parent.absolute()
    return (script_dir / ".." / ".." / "..").resolve()


def read_toml(path: Path) -> dict[str, object]:
    """Read TOML file."""
    with path.open("rb") as f:
        return tomllib.load(f)


def get_workspace_deps(repo_root: Path) -> dict[str, object]:
    """Extract workspace.dependencies from root Cargo.toml."""
    cargo_toml_path = repo_root / "Cargo.toml"
    data = read_toml(cargo_toml_path)
    return data.get("workspace", {}).get("dependencies", {})


def get_workspace_version(repo_root: Path) -> str:
    """Extract version from workspace.package."""
    cargo_toml_path = repo_root / "Cargo.toml"
    data = read_toml(cargo_toml_path)
    return data.get("workspace", {}).get("package", {}).get("version", "0.0.0")


def get_workspace_package(repo_root: Path) -> dict[str, object]:
    """Extract workspace.package metadata."""
    cargo_toml_path = repo_root / "Cargo.toml"
    data = read_toml(cargo_toml_path)
    return data.get("workspace", {}).get("package", {})


def format_dependency(name: str, dep_spec: object) -> str:
    """Format a dependency spec for Cargo.toml."""
    if isinstance(dep_spec, str):
        return f'{name} = "{dep_spec}"'
    if isinstance(dep_spec, dict):
        version: str = dep_spec.get("version", "")
        package: str | None = dep_spec.get("package")
        features: list[str] = dep_spec.get("features", [])
        default_features: bool | None = dep_spec.get("default-features")

        parts: list[str] = []

        if package:
            parts.append(f'package = "{package}"')

        parts.append(f'version = "{version}"')

        if features:
            features_str = ", ".join(f'"{f}"' for f in features)
            parts.append(f"features = [{features_str}]")

        if default_features is False:
            parts.append("default-features = false")
        elif default_features is True:
            parts.append("default-features = true")

        spec_str = ", ".join(parts)
        return f"{name} = {{ {spec_str} }}"

    return f'{name} = "{dep_spec}"'


def _make_field_replacer(dep_name: str, dep_spec: object) -> Callable[[re.Match[str]], str]:
    """Create a replacer function for workspace deps with extra fields."""
    base_spec = format_dependency(dep_name, dep_spec)

    def replace_with_fields(match: re.Match[str]) -> str:
        other_fields_str = match.group(1).strip()
        if " = { " not in base_spec:
            version_val = base_spec.split(" = ", 1)[1].strip('"')
            spec_part = f'version = "{version_val}"'
        else:
            spec_part = base_spec.split(" = { ", 1)[1].rstrip("}")

        existing_keys: set[str] = set()
        for raw_part in spec_part.split(","):
            stripped = raw_part.strip()
            if "=" in stripped:
                key = stripped.split("=")[0].strip()
                existing_keys.add(key)

        filtered_fields: list[str] = []
        for raw_field in other_fields_str.split(","):
            stripped_field = raw_field.strip()
            if stripped_field and "=" in stripped_field:
                key = stripped_field.split("=")[0].strip()
                if key not in existing_keys:
                    filtered_fields.append(stripped_field)
            elif stripped_field:
                filtered_fields.append(stripped_field)

        if filtered_fields:
            return f"{dep_name} = {{ {spec_part}, {', '.join(filtered_fields)} }}"
        return f"{dep_name} = {{ {spec_part} }}"

    return replace_with_fields


def replace_workspace_deps_in_toml(toml_path: Path, workspace_deps: dict[str, object]) -> None:
    """Replace workspace = true with explicit versions in a Cargo.toml file."""
    content = toml_path.read_text()

    for name, dep_spec in workspace_deps.items():
        # Simple: dep = { workspace = true }
        pattern1 = rf"^{re.escape(name)} = \{{ workspace = true \}}$"
        content = re.sub(pattern1, format_dependency(name, dep_spec), content, flags=re.MULTILINE)

        # With extra fields: dep = { workspace = true, optional = true }
        pattern2 = rf"^{re.escape(name)} = \{{ workspace = true, (.+?) \}}$"
        content = re.sub(
            pattern2,
            _make_field_replacer(name, dep_spec),
            content,
            flags=re.MULTILINE | re.DOTALL,
        )

    toml_path.write_text(content)


def generate_vendor_cargo_toml(
    vendor_dir: Path,
    workspace_deps: dict[str, object],
    pkg: dict[str, object],
    copied_crates: list[str],
) -> None:
    """Generate vendor/Cargo.toml with workspace setup."""
    deps_lines: list[str] = []
    for name, dep_spec in sorted(workspace_deps.items()):
        deps_lines.append(format_dependency(name, dep_spec))

    deps_str = "\n".join(deps_lines)
    members_str = ", ".join(f'"{m}"' for m in copied_crates)

    version = pkg.get("version", "0.0.0")
    edition = pkg.get("edition", "2024")
    license_val = pkg.get("license", "MIT OR Apache-2.0")
    repository = pkg.get("repository", "")

    vendor_toml = f"""[workspace]
members = [{members_str}]

[workspace.package]
version = "{version}"
edition = "{edition}"
license = "{license_val}"
repository = "{repository}"

[workspace.dependencies]
{deps_str}
"""

    vendor_dir.mkdir(parents=True, exist_ok=True)
    (vendor_dir / "Cargo.toml").write_text(vendor_toml)


def _clean_vendor_dir(vendor_base: Path) -> None:
    """Clean vendor directory, removing existing crate directories and Cargo.toml."""
    crate_names = ["ts-pack-core"]
    for name in crate_names:
        crate_path = vendor_base / name
        if crate_path.exists():
            shutil.rmtree(crate_path)
    vendor_cargo = vendor_base / "Cargo.toml"
    if vendor_cargo.exists():
        vendor_cargo.unlink()
    print("Cleaned vendor crate directories")


def _copy_crates(repo_root: Path, vendor_base: Path) -> list[str]:
    """Copy source crates into vendor directory."""
    crates_to_copy: list[tuple[str, str]] = [
        ("crates/ts-pack-core", "ts-pack-core"),
    ]

    copied_crates: list[str] = []
    for src_rel, dest_name in crates_to_copy:
        src: Path = repo_root / src_rel
        dest: Path = vendor_base / dest_name
        if src.exists():
            shutil.copytree(src, dest, ignore=shutil.ignore_patterns("target", "*.swp", "*.bak", "*.tmp", "*~"))
            copied_crates.append(dest_name)
            print(f"Copied {dest_name}")
        else:
            print(f"Warning: Source directory not found: {src_rel}")
    return copied_crates


def _clean_build_artifacts(vendor_base: Path, copied_crates: list[str]) -> None:
    """Remove build artifacts from copied crates."""
    for crate_dir in copied_crates:
        crate_path: Path = vendor_base / crate_dir
        if crate_path.exists():
            for artifact_dir in ["target", ".fastembed_cache"]:
                artifact: Path = crate_path / artifact_dir
                if artifact.exists():
                    shutil.rmtree(artifact)
    print("Cleaned build artifacts")


def _replace_workspace_inheritance(
    content: str,
    core_version: str,
    pkg: dict[str, object],
) -> str:
    """Replace workspace inheritance fields with explicit values."""
    replacements: list[tuple[str, str]] = [
        (r"^version\.workspace = true$", f'version = "{core_version}"'),
        (r"^edition\.workspace = true$", f'edition = "{pkg.get("edition", "2024")}"'),
        (r"^license\.workspace = true$", f'license = "{pkg.get("license", "MIT OR Apache-2.0")}"'),
        (r"^repository\.workspace = true$", f'repository = "{pkg.get("repository", "")}"'),
    ]
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    return content


def _update_copied_cargo_tomls(
    vendor_base: Path,
    copied_crates: list[str],
    workspace_deps: dict[str, object],
    core_version: str,
    pkg: dict[str, object],
) -> None:
    """Update workspace inheritance in copied Cargo.toml files."""
    for crate_dir in copied_crates:
        crate_toml = vendor_base / crate_dir / "Cargo.toml"
        if crate_toml.exists():
            content = crate_toml.read_text()
            content = _replace_workspace_inheritance(content, core_version, pkg)
            # Rename vendored package to avoid conflicts with workspace crate
            content = re.sub(
                r'^name = "ts-pack-core"$',
                'name = "ts-pack-core-vendored"',
                content,
                flags=re.MULTILINE,
            )
            crate_toml.write_text(content)
            replace_workspace_deps_in_toml(crate_toml, workspace_deps)
            print(f"Updated {crate_dir}/Cargo.toml")


def _update_php_cargo_toml(php_dir: Path) -> None:
    """Update PHP crate's Cargo.toml to point to vendored core."""
    php_cargo_toml = php_dir / "Cargo.toml"
    if php_cargo_toml.exists():
        content = php_cargo_toml.read_text()
        # Replace any existing ts-pack-core dependency line with the vendored path
        content = re.sub(
            r"^ts-pack-core = \{.*\}$",
            'ts-pack-core = { package = "ts-pack-core-vendored", path = "vendor/ts-pack-core" }',
            content,
            flags=re.MULTILINE,
        )
        php_cargo_toml.write_text(content)
        print("Updated ts-pack-php/Cargo.toml to use vendored core")


def main() -> None:
    """Main vendoring function."""
    repo_root: Path = get_repo_root()

    print("=== Vendoring ts-pack-core into PHP package ===")

    workspace_deps: dict[str, object] = get_workspace_deps(repo_root)
    pkg: dict[str, object] = get_workspace_package(repo_root)
    core_version: str = get_workspace_version(repo_root)

    print(f"Core version: {core_version}")
    print(f"Workspace dependencies: {len(workspace_deps)}")

    php_dir: Path = repo_root / "crates" / "ts-pack-php"
    vendor_base: Path = php_dir / "vendor"

    _clean_vendor_dir(vendor_base)
    vendor_base.mkdir(parents=True, exist_ok=True)

    copied_crates = _copy_crates(repo_root, vendor_base)
    _clean_build_artifacts(vendor_base, copied_crates)
    _update_copied_cargo_tomls(vendor_base, copied_crates, workspace_deps, core_version, pkg)

    generate_vendor_cargo_toml(vendor_base, workspace_deps, pkg, copied_crates)
    print("Generated vendor/Cargo.toml")

    _update_php_cargo_toml(php_dir)

    print(f"\nVendoring complete (core version: {core_version})")
    print(f"Copied crates: {', '.join(sorted(copied_crates))}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
