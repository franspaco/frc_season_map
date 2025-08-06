#!/usr/bin/env python3
"""
JSON Validation Script for FRC Season Map
Validates all JSON files in the repository to prevent syntax errors.
"""

import json
import sys
from pathlib import Path
import argparse


def validate_json_file(file_path):
    """Validate a single JSON file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            json.load(f)
        return True, None
    except json.JSONDecodeError as e:
        return False, str(e)
    except Exception as e:
        return False, f"Error reading file: {str(e)}"


def find_json_files(root_dir):
    """Find all JSON files in the repository, excluding .git directory."""
    root_path = Path(root_dir)
    json_files = []
    
    for file_path in root_path.rglob("*.json"):
        # Skip files in .git directory
        if ".git" not in file_path.parts:
            json_files.append(file_path)
    
    return sorted(json_files)


def main():
    parser = argparse.ArgumentParser(description="Validate JSON files in the FRC Season Map repository")
    parser.add_argument("--directory", "-d", default=".", help="Directory to search for JSON files")
    parser.add_argument("--quiet", "-q", action="store_true", help="Only show errors and summary")
    args = parser.parse_args()
    
    root_dir = Path(args.directory).resolve()
    
    if not args.quiet:
        print(f"🔍 Validating JSON files in: {root_dir}")
        print()
    
    json_files = find_json_files(root_dir)
    
    if not json_files:
        print("No JSON files found.")
        return 0
    
    valid_count = 0
    invalid_files = []
    
    for file_path in json_files:
        if not args.quiet:
            print(f"Checking: {file_path.relative_to(root_dir)}")
        
        is_valid, error_msg = validate_json_file(file_path)
        
        if is_valid:
            valid_count += 1
            if not args.quiet:
                print(f"✅ Valid")
        else:
            invalid_files.append((file_path, error_msg))
            if not args.quiet:
                print(f"❌ Invalid JSON")
                print(f"   Error: {error_msg}")
        
        if not args.quiet:
            print()
    
    # Summary
    total_files = len(json_files)
    
    if invalid_files:
        print(f"💥 Validation failed: {len(invalid_files)} out of {total_files} JSON files are invalid")
        print()
        print("Invalid files:")
        for file_path, error_msg in invalid_files:
            print(f"   - {file_path.relative_to(root_dir)}: {error_msg}")
        print()
        print("Please fix the JSON syntax errors.")
        return 1
    else:
        print(f"🎉 All {total_files} JSON files are valid!")
        return 0


if __name__ == "__main__":
    sys.exit(main())