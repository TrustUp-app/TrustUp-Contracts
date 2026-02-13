# Development Documentation

Guides and workflows for developing TrustUp smart contracts.

## Contents

- [Setup](setup.md) - Initial setup and installation
- [Workflow](workflow.md) - Development workflow and git practices
- [Testing](testing.md) - Testing strategies and patterns
- [Tools](tools.md) - Development tools and utilities

## Quick Start

```bash
# Clone repository
git clone https://github.com/yourusername/TrustUp-Contracts.git
cd TrustUp-Contracts

# Install dependencies
rustup target add wasm32-unknown-unknown

# Verify setup
cargo check
cargo test

# Build WASM
cargo build --target wasm32-unknown-unknown --release
```

## Development Cycle

1. **Pick Issue**: Choose from [Roadmap](../../ROADMAP.md)
2. **Create Branch**: `git checkout -b feat/SC-XX-description`
3. **Develop**: Write code following [standards](../standards/)
4. **Test**: Write comprehensive tests
5. **Review**: Run checks (fmt, clippy, test, build)
6. **Commit**: Atomic commits with conventional format
7. **Push**: Create PR with template
8. **Merge**: After approval and CI pass

## Common Commands

```bash
# Development
cargo check              # Quick compilation check
cargo build              # Native build
cargo test               # Run tests
cargo fmt                # Format code
cargo clippy             # Lint code

# WASM Build
cargo build -p <contract> --target wasm32-unknown-unknown --release

# Testing
cargo test -p <contract>           # Test specific contract
cargo test <test_name>             # Run specific test
cargo test -- --nocapture          # Show println output
```

## See Also

- [Contributing Guide](../../CONTRIBUTING.md) - Contribution guidelines
- [Standards](../standards/) - Code standards and conventions
- [Architecture](../architecture/) - System architecture
- [Resources](../resources/) - External tools and references
