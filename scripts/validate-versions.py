#!/usr/bin/env python3
"""
Version Consistency Validation Script for LazyCelery

This script validates that all version references across the project are consistent
and prevents version drift between different configuration files.

Usage:
    python3 scripts/validate-versions.py [--fix]
    
Options:
    --fix    Attempt to automatically fix version inconsistencies
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional


class VersionValidator:
    """Validates version consistency across project files"""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.errors: List[str] = []
        self.warnings: List[str] = []
        
    def parse_toml_simple(self, content: str) -> Dict[str, str]:
        """Simple TOML parser for extracting version information"""
        lines = content.split('\n')
        in_package = False
        data = {}
        
        for line in lines:
            line = line.strip()
            if line == '[package]':
                in_package = True
                continue
            elif line.startswith('[') and line != '[package]':
                in_package = False
                continue
                
            if in_package and '=' in line:
                key, value = line.split('=', 1)
                key = key.strip()
                value = value.strip().strip('"\'')
                data[key] = value
                
        return data
    
    def get_cargo_version(self) -> Optional[str]:
        """Get version from Cargo.toml"""
        cargo_toml = self.project_root / "Cargo.toml"
        if not cargo_toml.exists():
            self.errors.append("Cargo.toml not found")
            return None
            
        with open(cargo_toml, 'r') as f:
            content = f.read()
            
        package_data = self.parse_toml_simple(content)
        return package_data.get('version')
    
    def get_cargo_rust_version(self) -> Optional[str]:
        """Get rust-version from Cargo.toml"""
        cargo_toml = self.project_root / "Cargo.toml"
        if not cargo_toml.exists():
            return None
            
        with open(cargo_toml, 'r') as f:
            content = f.read()
            
        package_data = self.parse_toml_simple(content)
        return package_data.get('rust-version')
    
    def get_mise_rust_version(self) -> Optional[str]:
        """Get Rust version from .mise.toml"""
        mise_toml = self.project_root / ".mise.toml"
        if not mise_toml.exists():
            return None
            
        with open(mise_toml, 'r') as f:
            content = f.read()
            
        # Extract rust version from [tools] section
        lines = content.split('\n')
        in_tools = False
        
        for line in lines:
            line = line.strip()
            if line == '[tools]':
                in_tools = True
                continue
            elif line.startswith('[') and line != '[tools]':
                in_tools = False
                continue
                
            if in_tools and 'rust' in line and '=' in line:
                _, value = line.split('=', 1)
                value = value.strip().strip('"\'')
                return value
                
        return None
    
    def get_dockerfile_rust_version(self) -> Optional[str]:
        """Get Rust version from Dockerfile"""
        dockerfile = self.project_root / "Dockerfile"
        if not dockerfile.exists():
            return None
            
        with open(dockerfile, 'r') as f:
            content = f.read()
            
        # Look for FROM rust:VERSION pattern
        match = re.search(r'FROM rust:([0-9.]+)', content)
        return match.group(1) if match else None
    
    def validate_rust_versions(self) -> bool:
        """Validate that all Rust versions are consistent"""
        cargo_rust = self.get_cargo_rust_version()
        mise_rust = self.get_mise_rust_version()
        docker_rust = self.get_dockerfile_rust_version()
        
        all_good = True
        
        if not cargo_rust:
            self.errors.append("No rust-version found in Cargo.toml")
            all_good = False
            
        if not mise_rust:
            self.errors.append("No Rust version found in .mise.toml")
            all_good = False
            
        if not docker_rust:
            self.warnings.append("No Rust version found in Dockerfile")
        
        # Compare versions if all are present
        if cargo_rust and mise_rust:
            if cargo_rust != mise_rust:
                self.errors.append(f"Rust version mismatch: Cargo.toml={cargo_rust}, .mise.toml={mise_rust}")
                all_good = False
                
        if cargo_rust and docker_rust:
            if cargo_rust != docker_rust:
                self.errors.append(f"Rust version mismatch: Cargo.toml={cargo_rust}, Dockerfile={docker_rust}")
                all_good = False
                
        if mise_rust and docker_rust:
            if mise_rust != docker_rust:
                self.errors.append(f"Rust version mismatch: .mise.toml={mise_rust}, Dockerfile={docker_rust}")
                all_good = False
        
        return all_good
    
    def validate_package_versions(self) -> bool:
        """Validate that all package manager files have consistent versions"""
        cargo_version = self.get_cargo_version()
        if not cargo_version:
            self.errors.append("Could not determine version from Cargo.toml")
            return False
            
        all_good = True
        packaging_dir = self.project_root / "packaging"
        
        if not packaging_dir.exists():
            self.warnings.append("No packaging directory found")
            return True
        
        # Check various package files
        package_files = [
            ("packaging/chocolatey/lazycelery.nuspec", r'<version>([^<]+)</version>'),
            ("packaging/homebrew/lazycelery.rb", r'url ".*v([0-9.]+)\.tar\.gz"'),
            ("packaging/scoop/lazycelery.json", r'"version":\s*"([^"]+)"'),
            ("packaging/snap/snapcraft.yaml", r"version:\s*'([^']+)'"),
            ("packaging/aur/PKGBUILD", r'pkgver=([0-9.]+)'),
            ("packaging/aur/PKGBUILD-bin", r'pkgver=([0-9.]+)'),
        ]
        
        for file_path, pattern in package_files:
            full_path = self.project_root / file_path
            if not full_path.exists():
                self.warnings.append(f"Package file not found: {file_path}")
                continue
                
            with open(full_path, 'r') as f:
                content = f.read()
                
            match = re.search(pattern, content)
            if match:
                found_version = match.group(1)
                if found_version != cargo_version:
                    self.errors.append(f"Version mismatch in {file_path}: found={found_version}, expected={cargo_version}")
                    all_good = False
            else:
                self.warnings.append(f"Could not find version in {file_path}")
        
        return all_good
    
    def check_placeholder_hashes(self) -> bool:
        """Check for placeholder SHA256 hashes in package files"""
        all_good = True
        packaging_dir = self.project_root / "packaging"
        
        if not packaging_dir.exists():
            return True
            
        # Find all files that might contain SHA256 hashes
        for file_path in packaging_dir.rglob("*"):
            if file_path.is_file() and file_path.suffix in ['.rb', '.json', '.yaml', '.yml', '.nuspec']:
                with open(file_path, 'r') as f:
                    content = f.read()
                    
                if 'PLACEHOLDER_SHA256' in content:
                    self.warnings.append(f"Placeholder SHA256 found in {file_path.relative_to(self.project_root)}")
                    # This is a warning, not an error, as placeholders are expected during development
        
        return all_good
    
    def fix_rust_versions(self, target_version: str) -> bool:
        """Attempt to fix Rust version inconsistencies"""
        fixed_any = False
        
        # Fix Cargo.toml
        cargo_toml = self.project_root / "Cargo.toml"
        if cargo_toml.exists():
            with open(cargo_toml, 'r') as f:
                content = f.read()
                
            new_content = re.sub(
                r'rust-version = "[^"]*"',
                f'rust-version = "{target_version}"',
                content
            )
            
            if new_content != content:
                with open(cargo_toml, 'w') as f:
                    f.write(new_content)
                print(f"✅ Fixed Rust version in Cargo.toml to {target_version}")
                fixed_any = True
        
        # Fix Dockerfile
        dockerfile = self.project_root / "Dockerfile"
        if dockerfile.exists():
            with open(dockerfile, 'r') as f:
                content = f.read()
                
            new_content = re.sub(
                r'FROM rust:[0-9.]+',
                f'FROM rust:{target_version}',
                content
            )
            
            if new_content != content:
                with open(dockerfile, 'w') as f:
                    f.write(new_content)
                print(f"✅ Fixed Rust version in Dockerfile to {target_version}")
                fixed_any = True
        
        return fixed_any
    
    def fix_package_versions(self, target_version: str) -> bool:
        """Regenerate package files with correct version"""
        packaging_dir = self.project_root / "packaging"
        generate_script = packaging_dir / "generate_packages.py"
        
        if not generate_script.exists():
            print("❌ Cannot fix package versions: generate_packages.py not found")
            return False
            
        import subprocess
        try:
            result = subprocess.run([
                sys.executable, str(generate_script)
            ], cwd=self.project_root, capture_output=True, text=True)
            
            if result.returncode == 0:
                print("✅ Regenerated package files with correct versions")
                return True
            else:
                print(f"❌ Failed to regenerate packages: {result.stderr}")
                return False
                
        except Exception as e:
            print(f"❌ Error running package generator: {e}")
            return False
    
    def validate_all(self) -> bool:
        """Run all validations"""
        print("🔍 Validating version consistency across project...")
        
        rust_ok = self.validate_rust_versions()
        package_ok = self.validate_package_versions()
        self.check_placeholder_hashes()  # Only warnings
        
        return rust_ok and package_ok
    
    def print_results(self):
        """Print validation results"""
        if self.errors:
            print("\n❌ Errors found:")
            for error in self.errors:
                print(f"  • {error}")
                
        if self.warnings:
            print("\n⚠️  Warnings:")
            for warning in self.warnings:
                print(f"  • {warning}")
                
        if not self.errors and not self.warnings:
            print("\n✅ All version checks passed!")
        elif not self.errors:
            print("\n✅ No critical errors found (warnings only)")


def main():
    parser = argparse.ArgumentParser(description="Validate version consistency across project")
    parser.add_argument("--fix", action="store_true", help="Attempt to fix version inconsistencies")
    args = parser.parse_args()
    
    # Find project root
    script_dir = Path(__file__).parent.absolute()
    project_root = script_dir.parent
    
    validator = VersionValidator(project_root)
    
    # Run validation
    is_valid = validator.validate_all()
    
    # Attempt fixes if requested
    if args.fix and not is_valid:
        print("\n🔧 Attempting to fix version inconsistencies...")
        
        # Get the authoritative version from .mise.toml for Rust
        mise_rust = validator.get_mise_rust_version()
        cargo_version = validator.get_cargo_version()
        
        if mise_rust:
            validator.fix_rust_versions(mise_rust)
            
        if cargo_version:
            validator.fix_package_versions(cargo_version)
            
        # Re-validate after fixes
        print("\n🔍 Re-validating after fixes...")
        validator.errors.clear()
        validator.warnings.clear()
        is_valid = validator.validate_all()
    
    # Print results
    validator.print_results()
    
    # Exit with appropriate code
    sys.exit(0 if is_valid else 1)


if __name__ == "__main__":
    main()