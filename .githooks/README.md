# Git Hooks

This directory contains custom git hooks for the LazyCelery project.

## Setup

To enable the pre-commit hook, run:

```bash
git config core.hooksPath .githooks
```

This will configure git to use the hooks in this directory instead of `.git/hooks/`.

## Hooks

### pre-commit

Runs before each commit and performs the following checks:

1. **Code formatting** - Ensures code is properly formatted with `rustfmt`
2. **Linting** - Runs `clippy` to catch common issues and style violations
3. **Tests** - Runs the full test suite to ensure nothing is broken
4. **Security audit** - Checks for known security vulnerabilities in dependencies

If any check fails, the commit is aborted and you'll need to fix the issues before committing.

## Bypassing Hooks

In emergency situations, you can bypass the pre-commit hook with:

```bash
git commit --no-verify
```

However, this should be used sparingly as it defeats the purpose of having quality checks.

## Requirements

- `mise` must be installed and configured
- All mise tasks must be properly set up (`fmt`, `lint`, `test`, `audit`)