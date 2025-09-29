#!/bin/bash

# Nagari Repository SEO and Tags Setup Script
# Uses GitHub CLI to configure repository topics, description, and SEO settings

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub CLI path for Windows
GH_CMD="/c/Program Files/GitHub CLI/gh"

echo -e "${BLUE}🚀 Setting up Nagari Repository SEO and Tags${NC}"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "README.md" ]; then
    echo -e "${RED}❌ Error: Must be run from Nagari repository root${NC}"
    exit 1
fi

# Get current repository info
REPO_OWNER="ayanalamMOON"
REPO_NAME="Nagari"
REPO_FULL="${REPO_OWNER}/${REPO_NAME}"

echo -e "${YELLOW}📋 Repository: ${REPO_FULL}${NC}"

# 1. Update repository description
echo -e "\n${BLUE}📝 Updating repository description...${NC}"
NEW_DESCRIPTION="Modern programming language combining Python's elegant syntax with JavaScript's ecosystem compatibility. Rust-based transpiler for web development. React, Vue, Express compatible."

$GH_CMD repo edit $REPO_FULL --description "$NEW_DESCRIPTION"
echo -e "${GREEN}✅ Repository description updated${NC}"

# 2. Set repository homepage
echo -e "\n${BLUE}🏠 Setting repository homepage...${NC}"
$GH_CMD repo edit $REPO_FULL --homepage "https://github.com/${REPO_FULL}"
echo -e "${GREEN}✅ Repository homepage set${NC}"

# 3. Add comprehensive repository topics (tags)
echo -e "\n${BLUE}🏷️  Adding repository topics...${NC}"

# Primary topics - core functionality
PRIMARY_TOPICS=(
    "programming-language"
    "nagari" 
    "transpiler"
    "python-syntax"
    "javascript-interop"
    "rust-compiler"
    "typescript-runtime"
)

# Technology topics
TECH_TOPICS=(
    "rust"
    "typescript" 
    "javascript"
    "python"
    "webassembly"
    "nodejs"
    "browser"
    "es6"
)

# Development tools topics  
DEV_TOPICS=(
    "cli"
    "repl"
    "lsp"
    "vscode"
    "compiler"
    "lexer"
    "parser"
    "ast"
)

# Use case topics
USECASE_TOPICS=(
    "web-development"
    "react"
    "vue" 
    "express"
    "async-await"
    "type-checking"
    "package-manager"
    "cross-platform"
)

# Category topics
CATEGORY_TOPICS=(
    "developer-tools"
    "build-tools"
    "development-environment"
    "code-transpilation"
    "modern-javascript"
    "python-like"
    "functional-programming"
    "object-oriented"
)

# Community topics
COMMUNITY_TOPICS=(
    "open-source"
    "mit-license"
    "community-driven"
    "beginner-friendly"
    "well-documented"
    "production-ready"
)

# Combine all topics
ALL_TOPICS=(
    "${PRIMARY_TOPICS[@]}"
    "${TECH_TOPICS[@]}"
    "${DEV_TOPICS[@]}"
    "${USECASE_TOPICS[@]}"
    "${CATEGORY_TOPICS[@]}"
    "${COMMUNITY_TOPICS[@]}"
)

# GitHub has a limit of 20 topics, so we'll select the most important ones
SELECTED_TOPICS=(
    "programming-language"
    "nagari"
    "transpiler"
    "python-syntax"
    "javascript-interop"
    "rust-compiler"
    "web-development"
    "react"
    "vue"
    "express"
    "cli"
    "repl"
    "lsp"
    "developer-tools"
    "cross-platform"
    "open-source"
    "production-ready"
    "modern-javascript"
    "typescript"
    "nodejs"
)

echo -e "${YELLOW}Adding ${#SELECTED_TOPICS[@]} topics to repository...${NC}"

# Convert array to comma-separated string
TOPICS_STRING=$(IFS=','; echo "${SELECTED_TOPICS[*]}")

# Add topics to repository
$GH_CMD api repos/$REPO_FULL/topics -X PUT -f names="$TOPICS_STRING"

echo -e "${GREEN}✅ Repository topics added successfully${NC}"
echo -e "${BLUE}Topics added:${NC}"
for topic in "${SELECTED_TOPICS[@]}"; do
    echo "  • $topic"
done

# 4. Enable repository features for better SEO
echo -e "\n${BLUE}⚙️  Configuring repository features...${NC}"

# Enable issues if not already enabled
$GH_CMD repo edit $REPO_FULL --enable-issues=true
echo -e "${GREEN}✅ Issues enabled${NC}"

# Enable wiki if not already enabled  
$GH_CMD repo edit $REPO_FULL --enable-wiki=true
echo -e "${GREEN}✅ Wiki enabled${NC}"

# Enable projects if not already enabled
$GH_CMD repo edit $REPO_FULL --enable-projects=true
echo -e "${GREEN}✅ Projects enabled${NC}"

# Enable discussions if not already enabled
$GH_CMD repo edit $REPO_FULL --enable-discussions=true
echo -e "${GREEN}✅ Discussions enabled${NC}"

# 5. Create repository social preview image placeholder
echo -e "\n${BLUE}🎨 Creating social preview setup...${NC}"
if [ ! -d "assets" ]; then
    mkdir -p assets
    echo -e "${GREEN}✅ Created assets directory${NC}"
fi

# Create a placeholder file for the social preview image
cat > assets/social-preview-setup.md << 'EOL'
# Social Preview Image Setup

To complete SEO optimization, create a social preview image:

## Requirements:
- **Size**: 1280x640 pixels (2:1 ratio)
- **Format**: PNG or JPG
- **File size**: < 1MB
- **Filename**: `social-preview.png`

## Content Suggestions:
- Nagari logo and branding
- Code example showing Python-like syntax
- Key features: "Python Syntax + JavaScript Performance"
- Modern, professional design
- High contrast for readability

## Upload Instructions:
1. Create the image following the requirements above
2. Save as `assets/social-preview.png`
3. Upload to GitHub repository
4. Go to Settings > General > Social preview
5. Upload the image file

## Tools for Creation:
- Canva (canva.com)
- Figma (figma.com) 
- Adobe Photoshop
- GIMP (free alternative)

## Templates:
Look for "GitHub social preview" templates in design tools.
EOL

echo -e "${GREEN}✅ Social preview setup guide created${NC}"

# 6. Create issue templates for community engagement (if not exist)
echo -e "\n${BLUE}📋 Checking issue templates...${NC}"
if [ -d ".github/ISSUE_TEMPLATE" ]; then
    echo -e "${GREEN}✅ Issue templates already exist${NC}"
else
    echo -e "${YELLOW}⚠️  Issue templates not found - consider creating them for better community engagement${NC}"
fi

# 7. Display current repository stats
echo -e "\n${BLUE}📊 Current Repository Stats:${NC}"
$GH_CMD repo view $REPO_FULL --json stargazerCount,forkCount,watcherCount,openIssuesCount,description,topics | jq '{
    stars: .stargazerCount,
    forks: .forkCount, 
    watchers: .watcherCount,
    open_issues: .openIssuesCount,
    description: .description,
    topics: .topics
}'

# 8. Create a GitHub release for better SEO (if no releases exist)
echo -e "\n${BLUE}🏷️  Checking releases...${NC}"
RELEASE_COUNT=$($GH_CMD release list -R $REPO_FULL --json tagName | jq '. | length')

if [ "$RELEASE_COUNT" -eq "0" ]; then
    echo -e "${YELLOW}📦 No releases found. Creating initial release for SEO...${NC}"
    
    # Get current version from Cargo.toml
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    
    if [ -n "$VERSION" ]; then
        TAG_NAME="v$VERSION"
        RELEASE_TITLE="Nagari Programming Language v$VERSION"
        RELEASE_NOTES="# 🚀 Nagari Programming Language v$VERSION

## What is Nagari?

Nagari is a modern programming language that combines Python's elegant syntax with JavaScript's ecosystem compatibility. Built with Rust for performance and reliability.

### ✨ Key Features
- 🐍 **Python-inspired Syntax**: Clean, readable code with indentation-based structure
- ⚡ **JavaScript Performance**: Transpiles to optimized ES6+ with zero runtime overhead
- 🔧 **Complete Toolchain**: CLI, REPL, LSP, and package manager included
- 📦 **Universal Compatibility**: Works with React, Vue, Express, and npm packages
- 🎯 **Production Ready**: Successfully tested with real-world applications

### 🚀 Quick Start

\`\`\`bash
# Install runtime
npm install -g nagari-runtime

# Clone and build
git clone https://github.com/ayanalamMOON/Nagari.git
cd Nagari
cargo build --release

# Run your first program
echo 'print(\"Hello, Nagari!\")' > hello.nag
./target/release/nag run hello.nag
\`\`\`

### 📚 Documentation
- [Getting Started Guide](docs/getting-started.md)
- [Language Reference](docs/language-guide.md)
- [API Documentation](docs/api-reference.md)

### 🤝 Contributing
We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

**Built with ❤️ by the Nagari Team**"

        echo -e "${BLUE}Creating release $TAG_NAME...${NC}"
        $GH_CMD release create "$TAG_NAME" \
            --title "$RELEASE_TITLE" \
            --notes "$RELEASE_NOTES" \
            --latest \
            --generate-notes
        
        echo -e "${GREEN}✅ Initial release created: $TAG_NAME${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not determine version from Cargo.toml${NC}"
    fi
else
    echo -e "${GREEN}✅ Found $RELEASE_COUNT existing releases${NC}"
fi

# 9. Set up GitHub Pages (if applicable)
echo -e "\n${BLUE}🌐 Checking GitHub Pages setup...${NC}"
PAGES_INFO=$($GH_CMD api repos/$REPO_FULL/pages 2>/dev/null || echo "null")

if [ "$PAGES_INFO" = "null" ]; then
    echo -e "${YELLOW}📄 GitHub Pages not enabled${NC}"
    echo -e "${BLUE}To enable GitHub Pages for better SEO:${NC}"
    echo "1. Go to Settings > Pages"
    echo "2. Select source: Deploy from a branch"
    echo "3. Choose branch: main"
    echo "4. Folder: / (root)"
    echo "5. Save"
    echo "This will make your index.html accessible at: https://ayanalamMOON.github.io/Nagari"
else
    echo -e "${GREEN}✅ GitHub Pages already configured${NC}"
fi

# 10. Final SEO checklist
echo -e "\n${BLUE}📋 SEO Setup Complete! Next Steps:${NC}"
echo "============================================="
echo -e "${GREEN}✅ Repository description updated${NC}"
echo -e "${GREEN}✅ Homepage URL set${NC}"
echo -e "${GREEN}✅ Repository topics added (${#SELECTED_TOPICS[@]} topics)${NC}"
echo -e "${GREEN}✅ Repository features enabled${NC}"
echo -e "${GREEN}✅ Social preview setup guide created${NC}"

echo -e "\n${YELLOW}📌 Manual Tasks Remaining:${NC}"
echo "1. 🎨 Create and upload social preview image (1280x640px)"
echo "2. 🌐 Enable GitHub Pages in repository settings"
echo "3. 📊 Set up Google Analytics (add tracking ID to index.html)"
echo "4. 🔍 Submit sitemap to Google Search Console"
echo "5. 📱 Create social media accounts (@NagariLang)"
echo "6. 🏷️ Consider creating additional releases with binaries"

echo -e "\n${BLUE}🔗 Useful Links:${NC}"
echo "• Repository: https://github.com/$REPO_FULL"
echo "• Issues: https://github.com/$REPO_FULL/issues"
echo "• Discussions: https://github.com/$REPO_FULL/discussions"
echo "• Releases: https://github.com/$REPO_FULL/releases"
echo "• npm Package: https://www.npmjs.com/package/nagari-runtime"

echo -e "\n${GREEN}🎉 SEO setup completed successfully!${NC}"
echo -e "${BLUE}Your repository is now optimized for maximum discoverability! 🚀${NC}"

# Optional: Show current topics
echo -e "\n${BLUE}📋 Current Repository Topics:${NC}"
$GH_CMD api repos/$REPO_FULL/topics | jq -r '.names[]' | sed 's/^/  • /'

echo -e "\n${YELLOW}💡 Pro Tips:${NC}"
echo "• Regular releases improve SEO rankings"
echo "• Active issue management shows project health"
echo "• Documentation updates boost search visibility"
echo "• Community engagement increases repository score"
echo "• Social media promotion amplifies reach"