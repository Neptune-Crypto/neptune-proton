#!/bin/bash
# project-version.sh
# Manages the 'version' field in all Cargo.toml files within the project.

# --- Configuration ---
# Regex to find the version line, capturing the quotes and the version number.
# This pattern ensures we match "version = " followed by a semantic version string.
VERSION_EXTRACT_REGEX='^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)"'

GITHUB_REPOSITORY="Neptune-Crypto/neptune-proton";

# Global variable to store the new version for tagging
NEW_VERSION=""

# --- Utility Functions ---

# Function to display usage information
usage() {
    echo "Usage: $(basename "$0") [COMMAND]..."
    echo
    echo "Manages the 'version' string in all Cargo.toml files within the project."
    echo "Default action is check."
    echo
    echo "Commands (no hyphens required):"
    echo "  help                       Display this help message."
    echo "  check                      Report the 'version' string from all Cargo.toml files (default)."
    echo "  set <version-string>       Set a specific version (e.g., 1.2.3) in all files."
    echo "  bump <major|minor|point>   Increment the version number and update all files."
    echo "  changelog <version-string> Generate CHANGELOG.md content for the specified version."
    echo "  tag                        Create a Git tag (vX.Y.Z) for the newly set/bumped version."
    echo
    echo "Example: $(basename "$0") bump minor"
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

# ---------------------------------------------------------------------
# FINALIZED FUNCTION: action_changelog (FIXED BLANK LINE ISSUE)
# ---------------------------------------------------------------------
action_changelog() {
    local new_version="$1"

    # 1. Validation
    if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: changelog requires a valid version string (e.g., 1.2.3) as its argument." >&2
        return 1
    fi

    local output_file="CHANGELOG.md"
    local final_prepend_file=$(mktemp)

    # 2. Get the last documented tag from the file (e.g., v0.1.0)
    local last_doc_tag=""
    if [ -f "$output_file" ]; then
        # Find the most recent tag mentioned in the changelog file
        last_doc_tag=$(grep -E '^## v[0-9]+\.[0-9]+\.[0-9]+' "$output_file" | head -n 1 | sed -E 's/## (v[0-9]+\.[0-9]+\.[0-9]+)/\1/')
    fi

    # 3. Collect Tags to Process (in chronological/ascending order)
    local current_tag_name="v$new_version"
    local raw_tags
    local -a processing_tags=()

    # Get all tags (version sorted ascending: v0.1.0, v0.2.0, v0.3.0, ...)
    raw_tags=$(git tag --sort=version:refname --no-column --merged HEAD | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+')

    local start_filtering=false
    # Filter the list to include only tags newer than the last documented one.
    if [ -z "$last_doc_tag" ]; then
        start_filtering=true # Start from the beginning if no file exists
    fi

    while IFS= read -r tag; do
        if [ "$tag" == "$last_doc_tag" ]; then
            start_filtering=true
            continue # Skip the documented tag itself
        fi
        if $start_filtering; then
            processing_tags+=("$tag")
        fi
    done <<< "$raw_tags"

    # Ensure the new version tag is the final element, representing commits up to HEAD
    if [[ ! " ${processing_tags[@]} " =~ " ${current_tag_name} " ]]; then
        processing_tags+=("$current_tag_name")
    fi


    local -a generated_sections_files=() # Array to hold filenames of content sections
    local previous_tag_ref="$last_doc_tag"
    if [ -z "$previous_tag_ref" ]; then previous_tag_ref=""; fi
    local all_found=false


    # 4. Iterate and generate content for each release tag found (chronological order)
    for tag_name in "${processing_tags[@]}"; do

        local section_content_file=$(mktemp) # Temp file for this section

        local current_ref
        if [ "$tag_name" == "$current_tag_name" ]; then
            current_ref="HEAD"
        else
            current_ref="$tag_name"
        fi

        local tag_log_range
        if [ -n "$previous_tag_ref" ]; then
            tag_log_range="$previous_tag_ref..$current_ref"
        else
            tag_log_range="$current_ref"
        fi

        # Skip range if it's empty
        if [ "$previous_tag_ref" == "$current_ref" ]; then
             previous_tag_ref="$current_ref"
             rm -f "$section_content_file"
             continue
        fi

        # Print progress message
        if [ -n "$previous_tag_ref" ]; then
            echo "Generating changelog for $tag_name from $previous_tag_ref to $current_ref..."
        else
            echo "Generating changelog for $tag_name from beginning of history to $current_ref..."
        fi

        local all_found_in_section=false

        # Start the content block for this specific release
        echo "## $tag_name" >> "$section_content_file"
        echo "" >> "$section_content_file"

        # Define categories and their corresponding prefixes (FIXED CHORE EMOJI)
        local -a categories=(
            "🚀 Features;^feat(\(.*\))?:"
            "🔧 Fixes;^fix(\(.*\))?:"
            "📦 Build System;^build(\(.*\))?:"
            "⚙️  CI/CD;^ci(\(.*\))?:"
            "🛠️ Chore;^chore(\(.*\))?:"
            "📝 Documentation;^docs(\(.*\))?:"
            "♻️  Refactoring;^refactor(\(.*\))?:"
            "✅ Tests;^test(\(.*\))?:"
            "⚠️  Work in Progress;^wip(\(.*\))?:"
        )

        # List of all prefixes used for filtering uncategorized commits
        local all_prefixes="^feat(\(.*\))?:|^fix(\(.*\))?:|^build(\(.*\))?:|^ci(\(.*\))?:|^chore(\(.*\))?:|^docs(\(.*\))?:|^refactor(\(.*\))?:|^test(\(.*\))?:|^wip(\(.*\))?:"

        for cat_item in "${categories[@]}"; do
            IFS=';' read -r title regex_prefix <<< "$cat_item"

            local commits
            # Added -E for extended regex matching
            commits=$(git log -E "$tag_log_range" --no-merges --pretty=format:"* %s ([%h](https://github.com/${GITHUB_REPOSITORY}/commit/%H))" --grep="$regex_prefix")

            if [ -n "$commits" ]; then
                all_found_in_section=true
                echo "$title" >> "$section_content_file"
                echo "" >> "$section_content_file"
                # Remove redundant conventional commit prefix from the message body
                echo "$commits" | sed -E 's/^(\* \w+(\([^)]*\))?: )/\* /' >> "$section_content_file"
                echo "" >> "$section_content_file" # Blank line after commit list
            fi
        done

        # Handle Uncategorized commits (Inverse grep uses all prefixes collected)
        local uncategorized_commits
        # Added -E for extended regex matching
        uncategorized_commits=$(git log -E "$tag_log_range" --no-merges --pretty=format:"* %s ([%h](https://github.com/${GITHUB_REPOSITORY}/commit/%H))" --invert-grep --grep="$all_prefixes")

        if [ -n "$uncategorized_commits" ]; then
            all_found_in_section=true
            echo "Other Changes" >> "$section_content_file"
            echo "" >> "$section_content_file"
            echo "$uncategorized_commits" >> "$section_content_file"
            echo "" >> "$section_content_file" # Blank line after commit list
        fi

        # Remove the final trailing blank line from the temporary file content
        if [ -f "$section_content_file" ]; then
            # Remove all trailing blank lines, leaving only content
            # This is the critical step for controlling spacing
            sed -i.bak -e :a -e '/^\n*$/{$d;N;ba' -e '}' "$section_content_file"
            rm -f "$section_content_file.bak"
        fi


        # Store the filename in the array (still oldest-to-newest)
        generated_sections_files+=("$section_content_file")

        # Update the previous tag reference for the next iteration
        previous_tag_ref="$current_ref"
        if $all_found_in_section; then all_found=true; fi
    done # End of tag iteration loop


    # 5. Reverse the generated sections and write them to the final prepend file (Newest first)

    local i
    local is_first_section=true
    # Iterate BACKWARDS (reverse order) over the file array
    for ((i=${#generated_sections_files[@]}-1; i>=0; i--)); do
        local content_file="${generated_sections_files[i]}"

        # Read the content
        local content=$(cat "$content_file")

        if $is_first_section; then
            is_first_section=false
	else
            # FIX: Add a single, guaranteed blank line BEFORE the older release block
            echo "" >> "$final_prepend_file"
        fi

        # Write to the final prepend file (Newest at top)
        # Since content has no trailing newlines, echo adds exactly one trailing newline.
        echo "$content" >> "$final_prepend_file"

        rm -f "$content_file" # Clean up temp file
    done

    # 6. Finalize Files

    # Prepend the total content (all new sections) to the CHANGELOG.md file
    if [ -f "$output_file" ]; then

        # FIX: Check if the changelog file is NOT empty (i.e., has content other than the header),
        # and if so, add a blank line to separate the newly generated content
        # from the existing content at the top of the old file.
        if [ -s "$output_file" ]; then
            # The changelog file is NOT empty.
            echo "" >> "$final_prepend_file"
        fi

        # Prepend new content + optional blank line
        cat "$final_prepend_file" "$output_file" > "$output_file.tmp"
        mv "$output_file.tmp" "$output_file"
    else
        # Create new file with header
        echo "# Changelog" > "$output_file"
        cat "$final_prepend_file" >> "$output_file"
    fi

    rm -f "$final_prepend_file"

    # Final success message refers only to the files and action.
    echo "Success: Changelog content generated and prepended to $output_file."
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
    fi

    echo "Bumping version from $current_version to $new_version (Type: $bump_type)..."

    action_set "$new_version" || return 1
}

action_tag() {
    if [ -z "$NEW_VERSION" ]; then
        echo "Error: Cannot tag. Please run set or bump first in the same script execution." >&2
        return 1
    fi

    local tag_name="v$NEW_VERSION"

    echo "Creating Git tag '$tag_name'..."

    # Check for uncommitted changes (from set/bump and changelog)
    if ! git diff-index --quiet HEAD --; then
        echo "Committing version and changelog changes before tagging..."

        # Stage the files that were modified/created
        git add --update . # Adds modified Cargo.toml files
        git add CHANGELOG.md # Adds or updates the changelog

        git commit -m "build: Release $NEW_VERSION and update changelog" || {
            echo "Error: Failed to commit version and changelog changes." >&2
            return 1
        }
    fi

    # Tagging with force (-f) just in case the CI job is re-run
    git tag -f -a "$tag_name" -m "Release $NEW_VERSION" || {
        echo "Error: Failed to create or force git tag." >&2
        return 1
    }

    echo "Success: Tag '$tag_name' created."
    # The CI job will push the commit and the tag to the remote repository.
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
        help)
            usage
            ;;
        check)
            action_check
            exit $?
            ;;
        set)
            if [ -z "$2" ]; then
                echo "Error: set requires a version string argument." >&2
                exit 1
            fi
            action_set "$2"
            LAST_ACTION="set"
            shift 2
            ;;
        bump)
            if [ -z "$2" ]; then
                echo "Error: bump requires an argument (major, minor, or point)." >&2
                exit 1
            fi
            action_bump "$2" || exit 1
            LAST_ACTION="bump"
            shift 2
            ;;
        changelog)
            if [ -z "$2" ]; then
                echo "Error: changelog requires a version string argument (e.g., 1.2.3)." >&2
                exit 1
            fi
            action_changelog "$2" || exit 1
            LAST_ACTION="changelog"
            shift 2
            ;;
        tag)
            # Ensure the version is known before tagging
            if [ -z "$NEW_VERSION" ]; then
                NEW_VERSION=$(get_current_version)
                if [ $? -ne 0 ]; then exit 1; fi
            fi
            action_tag || exit 1
            shift
            ;;
        *)
            echo "Error: Unknown command '$1'. Use help for usage." >&2
            exit 1
            ;;
    esac
done

if [ $? -ne 0 ]; then
    exit 1
fi
