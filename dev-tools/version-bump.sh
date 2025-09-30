#!/bin/bash

# Nagari Version Management Tool
# Automated version bumping, tagging, and release preparation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
PACKAGE_JSON="$PROJECT_ROOT/nagari-runtime/package.json"
CHANGELOG="$PROJECT_ROOT/CHANGELOG.md"

# Version bump types
BUMP_TYPE=""
NEW_VERSION=""
DRY_RUN=false
AUTO_COMMIT=false
CREATE_TAG=false

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Nagari Version Management${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_version() {
    echo -e "${PURPLE}[VERSION]${NC} $1"
}

get_current_version() {
    if [ -f "$CARGO_TOML" ]; then
        grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
    else
        echo "0.1.0"
    fi
}

validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        print_error "Invalid version format: $version"
        print_info "Expected format: MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-PRERELEASE"
        return 1
    fi
    return 0
}

calculate_new_version() {
    local current_version="$1"
    local bump_type="$2"

    # Extract version parts
    local major=$(echo "$current_version" | cut -d. -f1)
    local minor=$(echo "$current_version" | cut -d. -f2)
    local patch=$(echo "$current_version" | cut -d. -f3 | cut -d- -f1)

    case "$bump_type" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            print_error "Invalid bump type: $bump_type"
            print_info "Valid types: major, minor, patch"
            return 1
            ;;
    esac

    echo "$major.$minor.$patch"
}

update_cargo_toml() {
    local new_version="$1"

    print_info "Updating Cargo.toml version to $new_version..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would update version in $CARGO_TOML"
        return 0
    fi

    # Create backup
    cp "$CARGO_TOML" "$CARGO_TOML.backup"

    # Update version
    sed -i.tmp "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    rm "$CARGO_TOML.tmp" 2>/dev/null || true

    print_success "Updated Cargo.toml"
}

update_package_json() {
    local new_version="$1"

    if [ ! -f "$PACKAGE_JSON" ]; then
        print_warning "package.json not found, skipping..."
        return 0
    fi

    print_info "Updating package.json version to $new_version..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would update version in $PACKAGE_JSON"
        return 0
    fi

    # Create backup
    cp "$PACKAGE_JSON" "$PACKAGE_JSON.backup"

    # Update version using node/npm if available
    if command -v npm &> /dev/null; then
        cd "$(dirname "$PACKAGE_JSON")"
        npm version "$new_version" --no-git-tag-version
        cd "$PROJECT_ROOT"
    else
        # Fallback: manual update
        sed -i.tmp "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
        rm "$PACKAGE_JSON.tmp" 2>/dev/null || true
    fi

    print_success "Updated package.json"
}

update_version_files() {
    local new_version="$1"

    # Update other version references
    local version_files=(
        "$PROJECT_ROOT/README.md"
        "$PROJECT_ROOT/docs/installation.md"
    )

    for file in "${version_files[@]}"; do
        if [ -f "$file" ]; then
            print_info "Updating version references in $(basename "$file")..."

            if ! $DRY_RUN; then
                # Update badge versions and similar references
                sed -i.tmp "s/version-[0-9]\+\.[0-9]\+\.[0-9]\+/version-$new_version/g" "$file"
                sed -i.tmp "s/v[0-9]\+\.[0-9]\+\.[0-9]\+/v$new_version/g" "$file"
                rm "$file.tmp" 2>/dev/null || true
                print_success "Updated $(basename "$file")"
            else
                print_info "[DRY RUN] Would update version references in $(basename "$file")"
            fi
        fi
    done
}

update_changelog() {
    local new_version="$1"
    local current_date=$(date '+%Y-%m-%d')

    if [ ! -f "$CHANGELOG" ]; then
        print_warning "CHANGELOG.md not found, creating..."
        if ! $DRY_RUN; then
            cat > "$CHANGELOG" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [$new_version] - $current_date

### Added
- Initial release

EOF
        fi
        return 0
    fi

    print_info "Updating CHANGELOG.md..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would update CHANGELOG.md with version $new_version"
        return 0
    fi

    # Create backup
    cp "$CHANGELOG" "$CHANGELOG.backup"

    # Create temporary file with updated changelog
    local temp_changelog=$(mktemp)

    # Read existing changelog and insert new version
    local unreleased_found=false
    while IFS= read -r line; do
        echo "$line" >> "$temp_changelog"

        if [[ "$line" == "## [Unreleased]" ]] && [ "$unreleased_found" = false ]; then
            echo "" >> "$temp_changelog"
            echo "## [$new_version] - $current_date" >> "$temp_changelog"
            echo "" >> "$temp_changelog"
            echo "### Added" >> "$temp_changelog"
            echo "- Version bump to $new_version" >> "$temp_changelog"
            echo "" >> "$temp_changelog"
            echo "### Changed" >> "$temp_changelog"
            echo "- Please update this section with actual changes" >> "$temp_changelog"
            echo "" >> "$temp_changelog"
            unreleased_found=true
        fi
    done < "$CHANGELOG"

    # Replace original changelog
    mv "$temp_changelog" "$CHANGELOG"

    print_success "Updated CHANGELOG.md"
    print_warning "Please edit CHANGELOG.md to add actual changes for this version"
}

check_working_directory() {
    if [ -d "$PROJECT_ROOT/.git" ]; then
        if ! git diff-index --quiet HEAD --; then
            print_warning "Working directory has uncommitted changes"
            if $AUTO_COMMIT; then
                print_info "Auto-commit enabled, will commit changes after version update"
            else
                print_info "Consider committing changes before version bump"
                read -p "Continue anyway? (y/N) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    print_info "Aborting version bump"
                    exit 0
                fi
            fi
        fi
    fi
}

run_tests() {
    print_info "Running tests before version bump..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would run tests"
        return 0
    fi

    cd "$PROJECT_ROOT"

    if cargo test --quiet; then
        print_success "All tests passed"
    else
        print_error "Tests failed. Fix tests before bumping version."
        return 1
    fi
}

commit_changes() {
    local new_version="$1"

    if [ ! -d "$PROJECT_ROOT/.git" ]; then
        print_warning "Not a git repository, skipping commit"
        return 0
    fi

    if ! $AUTO_COMMIT; then
        print_info "Auto-commit disabled, skipping commit"
        return 0
    fi

    print_info "Committing version changes..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would commit changes for version $new_version"
        return 0
    fi

    cd "$PROJECT_ROOT"

    # Add all changed files
    git add Cargo.toml
    [ -f "$PACKAGE_JSON" ] && git add "$PACKAGE_JSON"
    git add CHANGELOG.md
    git add README.md 2>/dev/null || true
    git add docs/ 2>/dev/null || true

    # Commit changes
    git commit -m "chore: bump version to $new_version

- Update Cargo.toml version
- Update package.json version (if exists)
- Update CHANGELOG.md
- Update version references in documentation

Release: v$new_version"

    print_success "Changes committed"
}

create_git_tag() {
    local new_version="$1"

    if [ ! -d "$PROJECT_ROOT/.git" ]; then
        print_warning "Not a git repository, skipping tag creation"
        return 0
    fi

    if ! $CREATE_TAG; then
        print_info "Tag creation disabled, skipping tag"
        return 0
    fi

    print_info "Creating git tag v$new_version..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would create tag v$new_version"
        return 0
    fi

    cd "$PROJECT_ROOT"

    # Create annotated tag
    git tag -a "v$new_version" -m "Release version $new_version

$(grep -A 10 "## \[$new_version\]" "$CHANGELOG" | tail -n +2 | head -n -1 || echo "Version $new_version release")"

    print_success "Created tag v$new_version"
    print_info "Push tag with: git push origin v$new_version"
}

show_summary() {
    local current_version="$1"
    local new_version="$2"

    echo
    echo -e "${GREEN}==== VERSION BUMP SUMMARY ====${NC}"
    echo -e "Previous version: ${YELLOW}$current_version${NC}"
    echo -e "New version:      ${GREEN}$new_version${NC}"
    echo -e "Bump type:        ${BLUE}$BUMP_TYPE${NC}"
    echo

    if $DRY_RUN; then
        echo -e "${YELLOW}DRY RUN MODE - No changes were made${NC}"
    else
        echo -e "${GREEN}Version bump completed successfully!${NC}"

        echo
        echo -e "${YELLOW}Next steps:${NC}"
        if $AUTO_COMMIT && [ -d "$PROJECT_ROOT/.git" ]; then
            echo "• Review the commit: git show HEAD"
        else
            echo "• Commit the changes: git add -A && git commit -m 'chore: bump version to $new_version'"
        fi

        if $CREATE_TAG && [ -d "$PROJECT_ROOT/.git" ]; then
            echo "• Push the tag: git push origin v$new_version"
        else
            echo "• Create and push tag: git tag v$new_version && git push origin v$new_version"
        fi

        echo "• Update CHANGELOG.md with actual changes"
        echo "• Run release preparation: ./dev-tools/release-prep.sh"
        echo "• Publish to registries (crates.io, npm, etc.)"
    fi
    echo
}

main() {
    print_header

    # Get current version
    local current_version=$(get_current_version)
    print_version "Current version: $current_version"

    # Calculate new version
    local new_version
    if [ -n "$NEW_VERSION" ]; then
        new_version="$NEW_VERSION"
        if ! validate_version "$new_version"; then
            exit 1
        fi
    elif [ -n "$BUMP_TYPE" ]; then
        new_version=$(calculate_new_version "$current_version" "$BUMP_TYPE")
    else
        print_error "No version or bump type specified"
        exit 1
    fi

    print_version "New version: $new_version"

    # Checks
    check_working_directory

    if ! $DRY_RUN; then
        run_tests
    fi

    # Update version files
    update_cargo_toml "$new_version"
    update_package_json "$new_version"
    update_version_files "$new_version"
    update_changelog "$new_version"

    # Git operations
    commit_changes "$new_version"
    create_git_tag "$new_version"

    # Show summary
    show_summary "$current_version" "$new_version"
}

# Help text
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Nagari Version Management Tool"
    echo
    echo "Automated version bumping, tagging, and release preparation."
    echo
    echo "Usage: $0 <bump-type|version> [options]"
    echo
    echo "Bump Types:"
    echo "  major              Bump major version (X.0.0)"
    echo "  minor              Bump minor version (0.X.0)"
    echo "  patch              Bump patch version (0.0.X)"
    echo
    echo "Or specify exact version:"
    echo "  1.2.3              Set specific version"
    echo
    echo "Options:"
    echo "  --dry-run          Show what would be changed without making changes"
    echo "  --commit           Automatically commit changes"
    echo "  --tag              Create git tag for the new version"
    echo "  --help, -h         Show this help message"
    echo
    echo "Examples:"
    echo "  $0 patch           Bump patch version"
    echo "  $0 minor --commit --tag    Bump minor version and create tag"
    echo "  $0 1.0.0 --dry-run         Preview setting version to 1.0.0"
    echo
    echo "Files updated:"
    echo "  • Cargo.toml"
    echo "  • nagari-runtime/package.json (if exists)"
    echo "  • CHANGELOG.md"
    echo "  • Version references in documentation"
    echo
    echo "Git operations (optional):"
    echo "  • Commit all version changes"
    echo "  • Create annotated git tag"
    exit 0
fi

# Parse arguments
if [ $# -eq 0 ]; then
    echo "Error: No arguments provided"
    echo "Use --help for usage information"
    exit 1
fi

# First argument is either bump type or version
case "$1" in
    "major"|"minor"|"patch")
        BUMP_TYPE="$1"
        shift
        ;;
    [0-9]*.[0-9]*.[0-9]*)
        NEW_VERSION="$1"
        shift
        ;;
    *)
        echo "Error: Invalid bump type or version: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac

# Parse remaining options
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --commit)
            AUTO_COMMIT=true
            shift
            ;;
        --tag)
            CREATE_TAG=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main "$@"
