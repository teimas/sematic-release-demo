#!/bin/bash

# TEIMAS Semantic Release - Git Commit Template Setup
# This script sets up the git commit template for consistent commit messages

echo "🚀 TEIMAS Semantic Release - Git Commit Template Setup"
echo "======================================================"

# Define the template path
TEMPLATE_PATH="$HOME/.gitmessage"

# Create the commit template
cat > "$TEMPLATE_PATH" << 'EOF'
# Commit Type and Scope
# Format: type(scope): subject
# Types: feat, fix, docs, style, refactor, perf, test, chore, revert
# Scope: Component or area affected (use N/A if not applicable)
type(scope): 

# Detailed Description
# Explain what and why vs how
# Use present tense: "change" not "changed" nor "changes"


# Breaking Changes
# List any breaking changes or N/A if none
BREAKING CHANGE: 

# Test Details  
# Describe testing performed or N/A if none
Test Details: 

# Security Considerations
# List security implications or N/A if none
Security: 

# Migraciones Lentas
# Describe slow migrations or N/A if none
Migraciones Lentas: 

# Partes a Ejecutar
# List deployment steps or N/A if none
Partes a Ejecutar: 

# Monday Tasks
# List related Monday.com tasks or N/A if none
# Format: - Task Title (ID: task_id) - Status
MONDAY TASKS: 


# ────────────────────────────────────────────────────────────────────────────
# COMMIT MESSAGE TEMPLATE GUIDELINES
# ────────────────────────────────────────────────────────────────────────────
#
# Subject Line (First Line):
# • Keep under 50 characters
# • Use imperative mood: "Add feature" not "Added feature" 
# • Don't end with a period
# • Be concise but descriptive
#
# Commit Types:
# • feat:     New feature for the user
# • fix:      Bug fix for the user
# • docs:     Documentation changes
# • style:    Code style changes (formatting, etc)
# • refactor: Code changes that neither fix bugs nor add features
# • perf:     Performance improvements
# • test:     Adding or fixing tests
# • chore:    Build process or auxiliary tools changes
# • revert:   Revert to a commit
#
# Body Guidelines:
# • Separate subject from body with a blank line
# • Use the body to explain what and why vs how
# • Each line should be under 72 characters
# • Use present tense: "change" not "changed" nor "changes"
#
# All Fields Required:
# • Use "N/A" for any field that doesn't apply
# • This ensures consistent commit structure across all commits
# • Makes automated parsing and analysis possible
#
# Examples:
# feat(auth): Add JWT authentication system
# fix(api): Resolve null pointer in user service  
# docs(readme): Update installation instructions
# test(user): Add unit tests for user validation
#
# Lines starting with # are comments and will be ignored
# ────────────────────────────────────────────────────────────────────────────
EOF

echo "📝 Created commit template at: $TEMPLATE_PATH"

# Prompt user for global vs local setup
echo ""
echo "Choose setup type:"
echo "1) Global (all repositories on this machine)"
echo "2) Local (current repository only)"
echo "3) Both"
echo ""
read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        git config --global commit.template "$TEMPLATE_PATH"
        echo "✅ Global git commit template configured"
        echo "💡 This will apply to all repositories on this machine"
        ;;
    2)
        if [ -d ".git" ]; then
            git config commit.template "$TEMPLATE_PATH"
            echo "✅ Local git commit template configured"
            echo "💡 This will apply only to the current repository"
        else
            echo "❌ Error: Not in a git repository. Please run this script from inside a git repository or choose global setup."
            exit 1
        fi
        ;;
    3)
        git config --global commit.template "$TEMPLATE_PATH"
        if [ -d ".git" ]; then
            git config commit.template "$TEMPLATE_PATH"
            echo "✅ Both global and local git commit templates configured"
        else
            echo "✅ Global git commit template configured"
            echo "⚠️  Local configuration skipped (not in a git repository)"
        fi
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again and choose 1, 2, or 3."
        exit 1
        ;;
esac

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📋 How to use:"
echo "• Run 'git commit' (without -m) to open editor with template"
echo "• Fill in the template fields, replacing placeholders with actual content"
echo "• Use 'N/A' for fields that don't apply"
echo "• The TEIMAS Semantic Release TUI will also follow this same structure"
echo ""
echo "🔧 To disable template:"
echo "• Global: git config --global --unset commit.template"
echo "• Local:  git config --unset commit.template"
echo ""
echo "✨ Happy committing with consistent messages!" 