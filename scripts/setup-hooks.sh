#!/usr/bin/env bash
# Setup script for Git hooks in Orlando project

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🔧 Setting up Git hooks for Orlando...${NC}\n"

# Get the root directory of the git repo
GIT_ROOT=$(git rev-parse --show-toplevel)

# Configure Git to use .githooks directory
echo -e "${YELLOW}Configuring Git to use .githooks directory...${NC}"
git config core.hooksPath .githooks

echo -e "${GREEN}✅ Git hooks configured!${NC}\n"

# Make hooks executable
echo -e "${YELLOW}Making hooks executable...${NC}"
chmod +x "$GIT_ROOT/.githooks"/*

echo -e "${GREEN}✅ Hooks are now executable${NC}\n"

# Show installed hooks
echo -e "${BLUE}📋 Installed hooks:${NC}"
ls -1 "$GIT_ROOT/.githooks" | grep -v "\.md$" || true

echo ""
echo -e "${GREEN}✨ Git hooks setup complete!${NC}"
echo ""
echo -e "${BLUE}ℹ️  The following hooks are now active:${NC}"
echo "  - pre-commit: Runs fmt, clippy, unit tests, integration tests, and build"
echo "  - pre-push: Runs comprehensive tests including property tests"
echo ""
echo -e "${YELLOW}💡 To skip hooks temporarily:${NC}"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""
echo -e "${YELLOW}💡 To disable hooks:${NC}"
echo "  git config --unset core.hooksPath"
echo ""
