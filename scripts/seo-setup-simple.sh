#!/bin/bash

# Simple SEO Setup Script for Nagari
# Uses GitHub CLI to add tags and configure repository

echo "🚀 Setting up Nagari Repository SEO and Tags"
echo "=============================================="

# Set variables
REPO="ayanalamMOON/Nagari"

# Update repository description
echo "📝 Updating repository description..."
gh repo edit $REPO --description "Modern programming language combining Python's elegant syntax with JavaScript's ecosystem compatibility. Rust-based transpiler for web development. React, Vue, Express compatible."

if [ $? -eq 0 ]; then
    echo "✅ Repository description updated"
else
    echo "❌ Failed to update description"
fi

# Set homepage
echo "🏠 Setting repository homepage..."
gh repo edit $REPO --homepage "https://github.com/$REPO"

if [ $? -eq 0 ]; then
    echo "✅ Repository homepage set"
else
    echo "❌ Failed to set homepage"
fi

# Add repository topics (max 20)
echo "🏷️ Adding repository topics..."

# Create JSON for topics
cat > topics.json << 'EOF'
{
  "names": [
    "programming-language",
    "nagari",
    "transpiler", 
    "python-syntax",
    "javascript-interop",
    "rust-compiler",
    "web-development",
    "react",
    "vue",
    "express",
    "cli",
    "repl",
    "lsp",
    "developer-tools",
    "cross-platform",
    "open-source",
    "production-ready",
    "modern-javascript",
    "typescript",
    "nodejs"
  ]
}
EOF

# Apply topics using the JSON file
gh api repos/$REPO/topics -X PUT --input topics.json

if [ $? -eq 0 ]; then
    echo "✅ Repository topics added successfully"
    echo "Topics added:"
    echo "  • programming-language"
    echo "  • nagari"
    echo "  • transpiler"
    echo "  • python-syntax"
    echo "  • javascript-interop"
    echo "  • rust-compiler"
    echo "  • web-development"
    echo "  • react"
    echo "  • vue"
    echo "  • express"
    echo "  • cli"
    echo "  • repl"
    echo "  • lsp"
    echo "  • developer-tools"
    echo "  • cross-platform"
    echo "  • open-source"
    echo "  • production-ready"
    echo "  • modern-javascript"
    echo "  • typescript"
    echo "  • nodejs"
else
    echo "❌ Failed to add topics"
fi

# Clean up temp file
rm -f topics.json

# Enable repository features
echo "⚙️ Enabling repository features..."

gh repo edit $REPO --enable-issues=true
echo "✅ Issues enabled"

gh repo edit $REPO --enable-wiki=true
echo "✅ Wiki enabled"

gh repo edit $REPO --enable-projects=true
echo "✅ Projects enabled"

gh repo edit $REPO --enable-discussions=true
echo "✅ Discussions enabled"

# Show current repository info
echo "📊 Current Repository Status:"
gh repo view $REPO

echo ""
echo "🎉 SEO Setup Complete!"
echo "======================"
echo "✅ Repository description updated"
echo "✅ Homepage URL set"
echo "✅ 20 SEO-optimized topics added"
echo "✅ Repository features enabled"
echo ""
echo "📌 Next Steps:"
echo "1. 🎨 Add social preview image (Settings > General > Social preview)"
echo "2. 🌐 Enable GitHub Pages for index.html"
echo "3. 📊 Set up Google Analytics"
echo "4. 📱 Create social media accounts"
echo ""
echo "🔗 Your repository: https://github.com/$REPO"