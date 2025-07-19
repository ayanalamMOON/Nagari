#!/bin/bash
# Release helper script for Nagari Programming Language
# Usage: ./scripts/release.sh [version]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}🔷 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Get version from argument or default
VERSION=${1:-"0.3.0"}
TAG="v${VERSION}"

print_step "Starting release process for Nagari ${VERSION}"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check if git is clean
if ! git diff-index --quiet HEAD --; then
    print_warning "Working directory is not clean. Commit your changes first."
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Run tests
print_step "Running tests"
cargo test --workspace
print_success "All tests passed"

# Build and test runtime
print_step "Building nagari-runtime"
cd nagari-runtime
npm install
npm run build
# Skip tests for now
cd ..
print_success "Runtime built successfully"

# Build binaries for local testing
print_step "Building release binaries"
cargo build --release --bin nag
cargo build --release --bin nagari-lsp
print_success "Binaries built successfully"

# Test CLI functionality
print_step "Testing CLI functionality"
./target/release/nag --version
echo 'print("Release test successful!")' > test_release.nag
# Test the binary
echo "Testing the binary..."
./target/release/nag build test_release.nag
rm -f test_release.nag test_release.js
print_success "CLI test passed"

# Update version in package.json
print_step "Updating version in nagari-runtime/package.json"
cd nagari-runtime
npm version ${VERSION} --no-git-tag-version
cd ..

# Check if CHANGELOG.md exists and has entry for this version
if [ -f "CHANGELOG.md" ]; then
    if grep -q "## \[${VERSION}\]" CHANGELOG.md; then
        print_success "CHANGELOG.md has entry for version ${VERSION}"
    else
        print_warning "CHANGELOG.md does not have entry for version ${VERSION}"
        echo "Please add a changelog entry before proceeding."
    fi
else
    print_warning "CHANGELOG.md not found. Consider creating one."
fi

# Create git tag
print_step "Creating git tag ${TAG}"
if git tag -l | grep -q "^${TAG}$"; then
    print_warning "Tag ${TAG} already exists"
    read -p "Delete existing tag and continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -d ${TAG}
        git push origin --delete ${TAG} 2>/dev/null || true
    else
        exit 1
    fi
fi

git tag -a ${TAG} -m "Release ${TAG}"
print_success "Tag ${TAG} created"

# Push tag to trigger release workflow
print_step "Pushing tag to GitHub"
git push origin ${TAG}
print_success "Tag pushed to GitHub"

print_step "Release process completed!"
echo
echo "🚀 The GitHub Actions workflow will now:"
echo "   • Build binaries for all platforms"
echo "   • Create a GitHub release"
echo "   • Upload release assets"
echo "   • Publish to npm (if configured)"
echo
echo "📋 Monitor the progress at:"
echo "   https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\)\.git/\1/')/actions"
echo
echo "🔗 Once complete, the release will be available at:"
echo "   https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\)\.git/\1/')/releases/tag/${TAG}"
