#!/bin/bash
# project-version.sh
# Manages the 'version' field in all Cargo.toml files within the project.

# --- Configuration ---
# Regex to find the version line, capturing the quotes and the version number.
# This pattern ensures we match "version = " followed by a semantic version string.
VERSION_EXTRACT_REGEX='^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)"'

# --- Utility Functions ---

# Function to display usage information
usage() {
    echo "Usage: $(basename "$0") [OPTION]..."
    echo
    echo "Manages the 'version' string in all Cargo.toml files within the project."
    echo "Default action is --check."
    echo
    echo "Options:"
    echo "  --help                       Display this help message."
    echo "  --check                      Report the 'version' string from all Cargo.toml files (default)."
    echo "  --set <version-string>       Set a specific version (e.g., 1.2.3) in all files."
    echo "  --bump <major|minor|point>   Increment the version number and update all files."
    echo "  --tag                        Create a Git tag (vX.Y.Z) for the newly set or bumped version (force if tag exists)."
    echo
    echo "Example: $(basename "$0") --bump minor"
    exit 0
}

# Function to find all Cargo.toml files and ensure a version is present.
find_cargo_files() {
    # Find files, excluding target/ directories for efficiency
    find . -type f -name "Cargo.toml" -not -path "./target/*"
}

# Function to safely extract the current version from the main package Cargo.toml.
get_current_version() {
    local first_file
    first_file=$(find_cargo_files | head -n 1)

    if [ -z "$first_file" ]; then
        echo "Error: No Cargo.toml files found." >&2
        return 1
    fi

    # Use awk to find the version line under the first [package] section encountered.
    local version_string
    version_string=$(awk '
        /^\s*\[package\]/ { in_package = 1; next }
        /^\s*\[/ { in_package = 0 } # Exit package section on next section
        /^\s*version\s*=/ && in_package {
            # Extract the quoted version string
            match($0, /"([^"]+)"/, version_match)
            print version_match[1]
            exit
        }' "$first_file")

    if [ -z "$version_string" ]; then
        echo "Error: Could not find 'version' under [package] in $first_file" >&2
        return 1
    fi

    echo "$version_string"
}

# Function to bump a version (MAJOR.MINOR.PATCH)
bump_version() {
    local current_version=$1
    local bump_type=$2
    local major minor point

    # Split version string using '.' as delimiter
    IFS='.' read -r major minor point <<< "$current_version"

    case "$bump_type" in
        major)
            major=$((major + 1))
            minor=0
            point=0
            ;;
        minor)
            minor=$((minor + 1))
            point=0
            ;;
        point)
            point=$((point + 1))
            ;;
        *)
            echo "Error: Invalid bump type '$bump_type'. Must be major, minor, or point." >&2
            return 1
            ;;
    esac

    echo "$major.$minor.$point"
}

# Function to perform the version update using sed
update_version_in_file() {
    local file=$1
    local new_version=$2

    # Escape the new version for safe use in sed substitution
    local sed_new_version
    sed_new_version=$(echo "$new_version" | sed 's/[\/&]/\\&/g')

    # The substitution pattern: find the line starting with 'version =' and replace
    # the entire version string (including quotes) with the new one.
    sed -i.bak -E "s/^version[[:space:]]*=[[:space:]]*\"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$sed_new_version\"/" "$file"

    if [ $? -ne 0 ]; then
        echo "Error: Failed to update version in $file." >&2
        rm -f "$file.bak"
        return 1
    fi

    # Clean up the backup file created by sed -i.bak (common on macOS)
    rm -f "$file.bak"
}

# --- Main Actions ---

action_check() {
    echo "--- Current Project Versions ---"
    
    local found_versions_count=0
    local -a file_details=() # Stores array of "version;filepath"
    declare -A version_map    # Key: version string, Value: 1 (for counting unique)

    # Process substitution avoids subshell scoping issues
    while read -r file; do
        local version_string

        # Extract version using the robust regex, ensuring only one version per file is captured
        version_string=$(sed -n -E "s/$VERSION_EXTRACT_REGEX/\1/p" "$file" | head -n 1)

        if [ -n "$version_string" ]; then
            # Store details for printing and mapping for consistency check
            file_details+=("$version_string;$file")
            version_map["$version_string"]=1
            found_versions_count=$((found_versions_count + 1))
        fi
    done < <(find_cargo_files)

    local unique_count="${#version_map[@]}"

    # 1. Print Detailed Output (Your original required format)
    for item in "${file_details[@]}"; do
        IFS=';' read -r version file <<< "$item"
        echo "$version in $file"
    done
    
    # Check if any files were processed
    if [ "$found_versions_count" -eq 0 ]; then
        echo "" # Blank line before summary
        echo "Status: ERROR"
        echo "Error: No 'version' field found in any Cargo.toml file."
        return 1
    fi

    # 2. Print Consistency Status (Your new requirement)
    echo "" # Blank line before summary
    if [ "$unique_count" -eq 1 ]; then
        local consistent_version
        for v in "${!version_map[@]}"; do consistent_version="$v"; break; done
        echo "Status: CONSISTENT (All $found_versions_count files match version $consistent_version)"
        return 0
    else
        echo "Status: INCONSISTENT (Found $unique_count conflicting versions)" >&2
        for version in "${!version_map[@]}"; do
            echo "Conflict: Version $version is present in one or more files." >&2
        done
        return 1
    fi
}

action_set() {
    local new_version=$1

    # Basic semantic versioning validation
    if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Invalid version format. Must be MAJOR.MINOR.POINT (e.g., 1.2.3)" >&2
        return 1
    fi

    echo "Setting version to $new_version in all Cargo.toml files..."
    local success=true
    find_cargo_files | while read -r file; do
        update_version_in_file "$file" "$new_version" || success=false
    done

    if $success; then
        echo "Success: All Cargo.toml files updated to $new_version."
        NEW_VERSION="$new_version"
    else
        echo "Error: Version update failed in one or more files." >&2
        return 1
    fi
}

action_bump() {
    local bump_type=$1

    # CRITICAL GATE: Check for consistency first. Fail if inconsistent.
    echo "Running consistency check before bumping..."
    if ! action_check; then
        echo "Error: Aborting bump operation. Cargo.toml files are inconsistent." >&2
        return 1
    fi
    
    # After check, get the consistent current version
    local current_version
    current_version=$(get_current_version)

    if [ $? -ne 0 ]; then
        # This should theoretically be caught by action_check, but kept for safety.
        return 1
    fi

    local new_version
    new_version=$(bump_version "$current_version" "$bump_type")

    if [ $? -ne 0 ]; then
        return 1
    fi # FIX: Closing the 'if' block with 'fi' instead of '}'

    echo "Bumping version from $current_version to $new_version (Type: $bump_type)..."

    action_set "$new_version" || return 1
}

action_tag() {
    if [ -z "$NEW_VERSION" ]; then
        echo "Error: Cannot tag. Please run --set or --bump first in the same script execution." >&2
        return 1
    fi

    local tag_name="v$NEW_VERSION"

    echo "Creating Git tag '$tag_name'..."

    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        echo "Committing version changes before tagging..."
        git commit -a -m "build: Bump project version to $NEW_VERSION" || {
            echo "Error: Failed to commit version changes." >&2
            return 1
        }
    fi

    # Tagging with force (-f) just in case the CI job is re-run
    git tag -f -a "$tag_name" -m "Release $NEW_VERSION" || {
        echo "Error: Failed to create or force git tag." >&2
        return 1
    }

    echo "Success: Tag '$tag_name' created."
    echo "Run 'git push --tags' to push the new tag to the remote repository."
}

# --- Script Execution ---

# Ensure we start in the correct directory (relative to the script location)
cd "$(dirname "$0")"/.. || exit 1

# If the user ran the script without arguments, ensure it defaults to check
if [ "$#" -eq 0 ]; then
    action_check
    exit $? # Exit with the status of action_check (0 for consistent, 1 for inconsistent)
fi

NEW_VERSION=""
LAST_ACTION=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --help)
            usage
            ;;
        --check)
            action_check
            exit $?
            ;;
        --set)
            if [ -z "$2" ]; then
                echo "Error: --set requires a version string argument." >&2
                exit 1
            fi
            action_set "$2"
            LAST_ACTION="set"
            shift 2
            ;;
        --bump)
            if [ -z "$2" ]; then
                echo "Error: --bump requires an argument (major, minor, or point)." >&2
                exit 1
            fi
            action_bump "$2"
            LAST_ACTION="bump"
            shift 2
            ;;
        --tag)
            # If --tag is used without a preceding --set or --bump,
            # we infer the version from the files and commit/tag immediately.
            if [ "$LAST_ACTION" != "set" ] && [ "$LAST_ACTION" != "bump" ]; then
                NEW_VERSION=$(get_current_version)
                if [ $? -ne 0 ]; then exit 1; fi
            fi
            action_tag
            shift
            ;;
        *)
            echo "Error: Unknown option or command '$1'. Use --help for usage." >&2
            exit 1
            ;;
    esac
done

if [ $? -ne 0 ]; then
    exit 1
fi
