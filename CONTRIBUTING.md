# Contributing to ic-rmcp

Thank you for your interest in contributing to ic-rmcp! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust (stable toolchain with `wasm32-unknown-unknown` target)
- Git
- Basic understanding of the [Model Context Protocol (MCP)](https://modelcontextprotocol.io)
- Familiarity with [Internet Computer](https://internetcomputer.org) development is helpful

### Development Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/ic-rmcp.git
   cd ic-rmcp
   ```

2. **Install the required Rust toolchain:**
   ```bash
   rustup target add wasm32-unknown-unknown
   rustup component add rustfmt clippy
   ```

3. **Build the project:**
   ```bash
   cargo build
   ```

4. **Run tests:**
   ```bash
   cargo test
   ```

5. **Check formatting and linting:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all --tests --no-deps -- -D warnings
   ```

## Making Changes

### Workflow

1. Create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following the guidelines below

3. Test your changes thoroughly

4. Commit your changes with descriptive commit messages

5. Push your branch and create a pull request

### Code Style

- Follow Rust naming conventions and idioms
- Use `cargo fmt` to format your code (configuration in `rustfmt.toml`)
- Ensure `cargo clippy` passes without warnings
- Write clear, self-documenting code with appropriate comments
- Add rustdoc comments for public APIs

### Commit Messages

Use clear and descriptive commit messages:

```
feat: add OAuth configuration support for MCP servers

- Add IssuerConfig and OAuthConfig types
- Implement OAuth token validation
- Update examples to demonstrate OAuth usage

Fixes #123
```

Format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions/changes
- `refactor:` for code refactoring
- `chore:` for maintenance tasks

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Add unit tests for new functionality
- Include integration tests for complex features
- Test both success and error cases
- Ensure tests work with the `wasm32-unknown-unknown` target

### Examples

Test your changes against the provided examples:

```bash
# Build examples
cargo build --examples

# Test specific example
cd examples/counter
cargo check
```

## Submitting Changes

### Pull Request Process

1. **Update documentation** if you've changed APIs or added features
2. **Add tests** for new functionality
3. **Ensure CI passes** - all checks must be green
4. **Update CHANGELOG** if applicable
5. **Request review** from maintainers

### Pull Request Template

Please include:

- **Description:** Clear explanation of what changes you made and why
- **Testing:** How you tested your changes
- **Documentation:** Any documentation updates needed
- **Breaking Changes:** Note any breaking changes
- **Related Issues:** Link to any related issues

## Reporting Bugs

### Before Reporting

1. **Search existing issues** to avoid duplicates
2. **Check the latest version** - your bug may already be fixed
3. **Test with minimal examples** to isolate the issue

### Bug Report Template

When reporting bugs, please include:

**Environment:**
- Rust version (`rustc --version`)
- ic-rmcp version
- Operating system
- Target platform (if relevant)

**Description:**
- Clear and concise description of the bug
- Expected behavior vs actual behavior

**Reproduction Steps:**
1. Step 1...
2. Step 2...
3. See error

**Code Sample:**
```rust
// Minimal reproducible example
```

**Error Output:**
```
// Include full error messages and stack traces
```

**Additional Context:**
- Screenshots (if applicable)
- Related issues or discussions
- Possible solutions you've tried

### Bug Report Labels

Issues will be labeled for triage:
- `bug` - Confirmed bugs
- `needs-reproduction` - Needs a minimal reproduction case
- `needs-investigation` - Requires further investigation
- `good-first-issue` - Suitable for new contributors
- `priority:high` - Critical bugs affecting security or core functionality

## Feature Requests

### Before Requesting

1. **Check existing issues** and discussions
2. **Consider the scope** - does it fit the project's goals?
3. **Think about implementation** - how might it work?

### Feature Request Template

**Problem Description:**
- What problem does this solve?
- Who would benefit from this feature?

**Proposed Solution:**
- Detailed description of the proposed feature
- How should it work?
- API design considerations

**Alternatives Considered:**
- Other approaches you've considered
- Why this approach is preferred

**Additional Context:**
- Use cases and examples
- Related features or issues
- Implementation complexity

## Documentation

### Types of Documentation

1. **API Documentation** - Rustdoc comments for public APIs
2. **Examples** - Working code examples in `examples/`
3. **README** - Getting started and overview
4. **Contributing Guide** - This document
5. **LLM Documentation** - `llms/llms-full.txt` for AI model consumption

### Documentation Guidelines

- Use clear, concise language
- Include code examples where helpful
- Keep examples up-to-date with API changes
- Test code examples to ensure they work
- Follow rustdoc conventions for API documentation

### Building Documentation

```bash
# Build documentation
cargo doc --no-deps -p ic-rmcp

# Build and open documentation
cargo doc --no-deps -p ic-rmcp --open
```

## Release Process

### Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Update documentation if needed
4. Test thoroughly including examples
5. Create release tag and GitHub release
6. Publish to crates.io (maintainers only)

## Getting Help

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and community discussion
- **YouTube Channel** - [ByteSmithLabs](https://www.youtube.com/@ByteSmithLabs) for tutorials

### Resources

- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Project Examples](./examples/)

## Maintainers

Current maintainers:
- Kyle ([lhkhiem@outlook.com](mailto:lhkhiem@outlook.com))
- Long Tran ([tranbaolong14@gmail.com](mailto:tranbaolong14@gmail.com))

## License

By contributing to ic-rmcp, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

Thank you for contributing to ic-rmcp! 🎉