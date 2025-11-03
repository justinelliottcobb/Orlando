# Git Hooks for Orlando

This directory contains Git hooks that help maintain code quality by running automated checks before commits and pushes.

## Available Hooks

### pre-commit
Runs before every commit. Performs:
- ✅ **rustfmt** - Code formatting check
- ✅ **clippy** - Linting and best practices
- ✅ **Unit tests** - Fast test suite
- ✅ **Documentation tests** - Validates code examples in docs
- ✅ **Integration tests** - End-to-end tests
- ✅ **Build check** - Ensures code compiles

**Estimated time:** 10-30 seconds

### pre-push
Runs before every push. Performs:
- ✅ **All tests** - Complete test suite
- ✅ **Property tests** - Algebraic property verification
- ✅ **Integration tests** - Full integration suite
- ✅ **Release build** - Optimized build verification

**Estimated time:** 30-60 seconds

## Setup

Run the setup script from the project root:

```bash
./scripts/setup-hooks.sh
```

Or manually:

```bash
git config core.hooksPath .githooks
chmod +x .githooks/*
```

## Usage

Once installed, hooks run automatically:

```bash
# pre-commit runs automatically
git commit -m "Your message"

# pre-push runs automatically
git push origin main
```

## Skipping Hooks

Sometimes you may need to skip hooks (use sparingly):

```bash
# Skip pre-commit
git commit --no-verify -m "Your message"

# Skip pre-push
git push --no-verify origin main
```

## Disabling Hooks

To completely disable hooks:

```bash
git config --unset core.hooksPath
```

To re-enable, run the setup script again.

## Customization

You can modify the hooks in this directory to suit your workflow:

- `.githooks/pre-commit` - Pre-commit checks
- `.githooks/pre-push` - Pre-push checks

After modifying, make sure they're executable:

```bash
chmod +x .githooks/*
```

## Troubleshooting

### Hooks not running
```bash
# Check hooks path configuration
git config core.hooksPath

# Should output: .githooks
# If not, run setup script again
./scripts/setup-hooks.sh
```

### Hooks failing unexpectedly
```bash
# Run checks manually to see detailed output
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --target x86_64-unknown-linux-gnu
cargo test --doc --target x86_64-unknown-linux-gnu
```

### Permission errors
```bash
# Make hooks executable
chmod +x .githooks/*
```

## CI/CD Integration

These hooks mirror the checks run in CI/CD (GitHub Actions), ensuring:
- Early feedback before pushing
- Fewer failed CI runs
- Faster development cycle
- Consistent code quality

## Best Practices

1. **Run hooks locally** - Don't skip them unless absolutely necessary
2. **Fix issues early** - Address formatting/linting before committing
3. **Keep commits focused** - Smaller commits = faster hook execution
4. **Update hooks** - Pull latest changes to stay in sync with team

## Performance Tips

If hooks are slow:

1. **Use `--quiet` flag** - Already enabled for tests
2. **Run specific tests** - Hooks skip heavy property tests by default
3. **Cache dependencies** - Cargo caches builds automatically
4. **Skip on WIP commits** - Use `--no-verify` for work-in-progress

## Hook Execution Order

1. **Developer commits** → pre-commit hook runs
2. **Commit succeeds** (if hook passes)
3. **Developer pushes** → pre-push hook runs
4. **Push succeeds** (if hook passes)
5. **CI/CD pipeline** → Runs comprehensive checks

## Questions?

See the main [TESTING.md](../TESTING.md) for more details on the test suite.
