#!/bin/bash
# Setup script to configure git hooks

set -e

echo "Setting up git hooks for flu..."

# Set git hooks path
git config core.hooksPath .githooks

echo "✅ Git hooks configured successfully!"
echo ""
echo "Pre-commit hooks will now run automatically:"
echo "  - Code formatting check (cargo fmt)"
echo "  - Linting (cargo clippy)"
echo "  - Unit tests (cargo test --lib)"
echo ""
echo "To skip hooks (not recommended), use: git commit --no-verify"
