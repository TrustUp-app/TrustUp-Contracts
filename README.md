# TrustUp Contracts

> Decentralized "Buy Now, Pay Later" (BNPL) platform on Stellar blockchain using Soroban smart contracts

[![Build Status](https://github.com/yourusername/TrustUp-Contracts/workflows/CI/badge.svg)](https://github.com/yourusername/TrustUp-Contracts/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What is TrustUp?

TrustUp enables users to make purchases by paying a 20% guarantee deposit upfront while receiving the remaining 80% as credit from a community-funded liquidity pool. The system uses **on-chain reputation** to reward good repayment behavior and penalize defaults.

### Key Features

- ✨ **Transparent Credit System**: All rules encoded in smart contracts
- 🔐 **Portable Reputation**: On-chain scores owned by users
- 💰 **Community Liquidity**: Decentralized pool of liquidity providers
- 🌍 **Financial Inclusion**: Accessible to anyone with a Stellar wallet
- ⚡ **Low Fees**: No middlemen, automated execution (~$0.00001 per transaction)

## 🏗️ Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ Reputation  │◄────┤  CreditLine  │────►│   Merchant   │
│  Contract   │     │   Contract   │     │   Registry   │
└─────────────┘     └──────────────┘     └──────────────┘
      ▲                     │
      │                     ▼
      │             ┌──────────────┐
      └─────────────┤  Liquidity   │
                    │     Pool     │
                    └──────────────┘
```

**Learn more**: [docs/architecture/](docs/architecture/)

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable)
- **Soroban SDK** (included via Cargo)
- **wasm32-unknown-unknown** target

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI (optional, for deployment)
cargo install stellar-cli --locked
```

### Clone and Build

```bash
# Clone repository
git clone https://github.com/yourusername/TrustUp-Contracts.git
cd TrustUp-Contracts

# Check compilation
cargo check

# Run tests
cargo test

# Build all contracts
cargo build --release

# Build WASM for deployment
cargo build -p reputation-contract --target wasm32-unknown-unknown --release
```

## 📦 Contracts

| Contract | Status | Description |
|----------|--------|-------------|
| **[Reputation](contracts/reputation-contract/)** | ✅ Complete | Manages user credit scores (0-100) |
| **[CreditLine](contracts/creditline-contract/)** | ⏳ In Progress | Handles loan creation and repayment |
| **[Merchant Registry](contracts/merchant-registry-contract/)** | ⏳ Planned | Whitelist of authorized merchants |
| **[Liquidity Pool](contracts/liquidity-pool-contract/)** | ⏳ Planned | Manages LP deposits and rewards |

### Reputation Contract ✅

Track and update user credit scores with role-based access control.

**Key Functions**:
```rust
pub fn get_score(env: Env, user: Address) -> u32
pub fn increase_score(env: Env, updater: Address, user: Address, amount: u32)
pub fn decrease_score(env: Env, updater: Address, user: Address, amount: u32)
```

**Features**:
- Score range: 0-100
- Admin and updater roles
- Event emission for all changes
- Comprehensive test coverage

## 🛠️ Development

### Project Structure

```
TrustUp-Contracts/
├── contracts/
│   ├── reputation-contract/        # ✅ User credit scores
│   ├── creditline-contract/        # ⏳ Loan management
│   ├── merchant-registry-contract/ # ⏳ Merchant whitelist
│   └── liquidity-pool-contract/    # ⏳ LP management
├── docs/                           # Comprehensive documentation
├── Cargo.toml                      # Workspace configuration
└── README.md                       # This file
```

### Common Commands

```bash
# Development
cargo check              # Quick compilation check
cargo test               # Run all tests
cargo fmt                # Format code
cargo clippy             # Lint code

# Building
cargo build              # Native build
cargo build --release    # Optimized build

# WASM Build (for deployment)
cargo build -p <contract-name> --target wasm32-unknown-unknown --release

# Example: Build reputation contract
cargo build -p reputation-contract --target wasm32-unknown-unknown --release
```

### Code Quality

We use automated tools to maintain code quality:

```bash
# Format check
cargo fmt -- --check

# Lint with warnings as errors
cargo clippy -- -D warnings

# Run tests with coverage
cargo test --verbose
```

## 📚 Documentation

Comprehensive documentation available in [`docs/`](docs/):

- **[Architecture](docs/architecture/)** - System design and contract architecture
- **[Standards](docs/standards/)** - Code standards and conventions
- **[Development](docs/development/)** - Development workflow and tools
- **[Resources](docs/resources/)** - External tools and references
  - [OpenZeppelin Tools](docs/resources/openzeppelin.md)
  - [Stellar & Soroban](docs/resources/stellar-soroban.md)
  - [AI Assistants & MCP](docs/resources/ai-assistants.md)

**Quick Links**:
- [Project Context](PROJECT_CONTEXT.md) - Vision and use cases
- [Roadmap](docs/ROADMAP.md) - Development timeline
- [Contributing Guide](CONTRIBUTING.md) - How to contribute

## 🤖 AI Development Tools

TrustUp integrates with modern AI development tools:

### OpenZeppelin Stellar Contracts

Configured in [`Cargo.toml`](contracts/reputation-contract/Cargo.toml):
```toml
[dependencies]
openzeppelin-stellar = { git = "https://github.com/OpenZeppelin/stellar-contracts" }
openzeppelin-soroban-helpers = { git = "https://github.com/OpenZeppelin/soroban-helpers" }
```

### Stellar MCP Server

MCP (Model Context Protocol) server for AI-assisted development with Claude.

**Setup**: See [docs/resources/ai-assistants.md](docs/resources/ai-assistants.md)

## 🧪 Testing

Comprehensive test suite with unit and integration tests.

```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p reputation-contract

# Run specific test
cargo test test_increase_score

# Show test output
cargo test -- --nocapture
```

**Test Coverage**: Each contract includes:
- ✅ Unit tests for all functions
- ✅ Error case testing
- ✅ Boundary value testing
- ✅ Access control testing
- ✅ Event emission verification

## 🔐 Security

Security is our top priority:

- ✅ Checked arithmetic (overflow/underflow protection)
- ✅ Authorization checks before state changes
- ✅ Input validation
- ✅ Event emission for auditability
- ✅ OpenZeppelin security patterns
- ⏳ External security audit (planned)

**Report vulnerabilities**: security@trustup.example (replace with actual contact)

## 🗺️ Roadmap

**Current Phase**: Phase 3 - CreditLine Contract Development

**Completed** ✅:
- Reputation Contract (8 issues)
- Access control and authorization
- Comprehensive test suite

**In Progress** ⏳:
- CreditLine Contract
- Loan creation and repayment logic
- Integration with Reputation contract

**Planned** 📋:
- Merchant Registry
- Liquidity Pool
- Full system integration tests

**See**: [docs/ROADMAP.md](docs/ROADMAP.md) for detailed breakdown

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Read**: [CONTRIBUTING.md](CONTRIBUTING.md)
2. **Pick an issue**: See [ROADMAP.md](docs/ROADMAP.md)
3. **Create branch**: `feat/SC-XX-description`
4. **Follow standards**: [docs/standards/](docs/standards/)
5. **Submit PR**: Use the PR template

### Development Workflow

```bash
# 1. Create feature branch
git checkout -b feat/SC-XX-my-feature

# 2. Make changes and test
cargo test
cargo fmt
cargo clippy

# 3. Commit with conventional commits
git commit -m "feat: implement loan creation (SC-08)"

# 4. Push and create PR
git push origin feat/SC-XX-my-feature
```

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## 🌟 Tech Stack

- **Blockchain**: [Stellar](https://stellar.org/)
- **Smart Contracts**: [Soroban](https://soroban.stellar.org/) (Rust → WASM)
- **SDK**: [soroban-sdk 22.0.0](https://docs.rs/soroban-sdk/)
- **Build Tool**: [Cargo](https://doc.rust-lang.org/cargo/)
- **Security**: [OpenZeppelin Stellar](https://github.com/OpenZeppelin/stellar-contracts)

## 🔗 Links

- **Documentation**: [docs/](docs/)
- **Project Context**: [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md)
- **Roadmap**: [docs/ROADMAP.md](docs/ROADMAP.md)
- **Issues**: [GitHub Issues](https://github.com/yourusername/TrustUp-Contracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/TrustUp-Contracts/discussions)

### Stellar Ecosystem

- [Stellar Developers](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Discord](https://discord.gg/stellar)
- [Stellar Expert](https://stellar.expert/) (Block Explorer)

### OpenZeppelin

- [OpenZeppelin Stellar](https://github.com/OpenZeppelin/stellar-contracts)
- [OpenZeppelin Docs](https://docs.openzeppelin.com/stellar-contracts)
- [Contract Wizard](https://wizard.openzeppelin.com/stellar)

## 💬 Community

- **Discord**: [Stellar Discord](https://discord.gg/stellar) - mention @TrustUp
- **GitHub**: [Issues](https://github.com/yourusername/TrustUp-Contracts/issues) and [Discussions](https://github.com/yourusername/TrustUp-Contracts/discussions)
- **Twitter**: [@TrustUp](https://twitter.com/trustup) (replace with actual handle)

## 📊 Status

**Version**: 1.0.0
**Status**: Active Development
**Last Updated**: February 2026

---

Built with ❤️ on [Stellar](https://stellar.org/) using [Soroban](https://soroban.stellar.org/)
