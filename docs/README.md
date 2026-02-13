# TrustUp Contracts Documentation

Comprehensive documentation for TrustUp smart contracts on Stellar/Soroban.

## 📚 Documentation Structure

### [Architecture](architecture/)
System architecture, contract design, and technical specifications.
- [Overview](architecture/overview.md) - System architecture and tech stack
- [Contracts](architecture/contracts.md) - Detailed contract architecture
- [Storage Patterns](architecture/storage-patterns.md) - Data storage strategies

### [Standards](standards/)
Code standards, conventions, and best practices.
- [Error Handling](standards/error-handling.md) - Error codes and patterns
- [File Organization](standards/file-organization.md) - Repository structure
- [Code Style](standards/code-style.md) - Rust code conventions
- [Naming Conventions](standards/naming-conventions.md) - Naming standards (coming soon)

### [Development](development/)
Development guides, workflows, and tools.
- [Setup](development/setup.md) - Initial setup instructions (coming soon)
- [Workflow](development/workflow.md) - Development process (coming soon)
- [Testing](development/testing.md) - Testing strategies (coming soon)
- [Tools](development/tools.md) - Development tools (coming soon)

### [Resources](resources/)
External resources, tools, and references.
- [OpenZeppelin](resources/openzeppelin.md) - OpenZeppelin tools and libraries
- [Stellar & Soroban](resources/stellar-soroban.md) - Platform documentation
- [AI Assistants & MCP](resources/ai-assistants.md) - AI development tools

## 🚀 Quick Start

### For New Contributors

1. **Read**: [PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md) - Understand TrustUp's vision
2. **Setup**: Install Rust and dependencies
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-unknown-unknown
   ```
3. **Clone**: Get the repository
   ```bash
   git clone https://github.com/yourusername/TrustUp-Contracts.git
   cd TrustUp-Contracts
   ```
4. **Verify**: Run tests
   ```bash
   cargo test
   ```
5. **Contribute**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

### For Developers

**Architecture Overview**: [architecture/overview.md](architecture/overview.md)

**Current Contracts**:
- ✅ [Reputation Contract](architecture/contracts.md#reputation-contract-) - User credit scores
- ⏳ [CreditLine Contract](architecture/contracts.md#creditline-contract-) - Loan management (in progress)
- ⏳ [Merchant Registry](architecture/contracts.md#merchant-registry-contract-) - Merchant whitelist (planned)
- ⏳ [Liquidity Pool](architecture/contracts.md#liquidity-pool-contract-) - LP management (planned)

**Development Roadmap**: [ROADMAP.md](ROADMAP.md)

### For Auditors

**Security Focus Areas**:
- Access control patterns: [architecture/overview.md](architecture/overview.md#authorization)
- Error handling: [standards/error-handling.md](standards/error-handling.md)
- Storage patterns: [architecture/storage-patterns.md](architecture/storage-patterns.md)
- Safe arithmetic: [standards/code-style.md](standards/code-style.md#error-handling)

**Test Coverage**: Run `cargo test` to see comprehensive test suite

## 📖 Key Documents

### Getting Started
- [PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md) - What is TrustUp?
- [CONTRIBUTING.md](../CONTRIBUTING.md) - How to contribute
- [ROADMAP.md](ROADMAP.md) - Development timeline

### Technical Reference
- [Architecture Overview](architecture/overview.md) - System design
- [Error Codes](standards/error-handling.md) - Complete error reference
- [Storage Patterns](architecture/storage-patterns.md) - Data management
- [Code Style](standards/code-style.md) - Coding standards

### Tools & Resources
- [OpenZeppelin Tools](resources/openzeppelin.md) - Security libraries
- [Stellar Platform](resources/stellar-soroban.md) - Blockchain platform
- [AI Assistants](resources/ai-assistants.md) - Development assistants

## 🛠️ Common Tasks

### Building Contracts

```bash
# Build all contracts (native)
cargo build --release

# Build WASM for deployment
cargo build -p reputation-contract --target wasm32-unknown-unknown --release
```

### Running Tests

```bash
# All tests
cargo test

# Specific contract
cargo test -p reputation-contract

# Specific test
cargo test test_increase_score
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check compilation
cargo check
```

## 🏗️ Project Structure

```
TrustUp-Contracts/
├── contracts/                          # Smart contracts
│   ├── reputation-contract/            # ✅ Reputation scoring
│   ├── creditline-contract/            # ⏳ Loan management
│   ├── merchant-registry-contract/     # ⏳ Merchant whitelist
│   └── liquidity-pool-contract/        # ⏳ LP management
│
├── docs/                               # Documentation (you are here)
│   ├── architecture/                   # System architecture
│   ├── standards/                      # Code standards
│   ├── development/                    # Dev workflows
│   └── resources/                      # External resources
│
├── Cargo.toml                          # Workspace configuration
├── CONTRIBUTING.md                     # Contribution guide
├── PROJECT_CONTEXT.md                  # Project vision
└── README.md                           # Project readme
```

## 📊 Contract Status

| Contract | Status | Description | Documentation |
|----------|--------|-------------|---------------|
| **Reputation** | ✅ Complete | User credit scores (0-100) | [Details](architecture/contracts.md#reputation-contract-) |
| **CreditLine** | ⏳ In Progress | Loan creation and repayment | [Details](architecture/contracts.md#creditline-contract-) |
| **Merchant Registry** | ⏳ Planned | Merchant whitelist | [Details](architecture/contracts.md#merchant-registry-contract-) |
| **Liquidity Pool** | ⏳ Planned | LP deposits and rewards | [Details](architecture/contracts.md#liquidity-pool-contract-) |

## 🔗 External Links

### Stellar & Soroban
- [Stellar Developers](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Soroban SDK Reference](https://docs.rs/soroban-sdk/)
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli)

### OpenZeppelin
- [OpenZeppelin Stellar Contracts](https://github.com/OpenZeppelin/stellar-contracts)
- [OpenZeppelin Documentation](https://docs.openzeppelin.com/stellar-contracts)
- [Contract Wizard](https://wizard.openzeppelin.com/stellar)

### Community
- [Stellar Discord](https://discord.gg/stellar) - #soroban channel
- [Stellar GitHub](https://github.com/stellar)
- [OpenZeppelin Forum](https://forum.openzeppelin.com/)

## 🤝 Contributing

We welcome contributions! Please see:
1. [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
2. [ROADMAP.md](ROADMAP.md) - Pick an issue to work on
3. [Code Style](standards/code-style.md) - Follow our standards

## 📝 License

This project is open source. See LICENSE file for details.

## 💬 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/TrustUp-Contracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/TrustUp-Contracts/discussions)
- **Discord**: [Stellar Discord](https://discord.gg/stellar) - mention @TrustUp

## 🗺️ Navigation

- **[← Back to Project](../README.md)**
- **[Architecture →](architecture/)**
- **[Standards →](standards/)**
- **[Development →](development/)**
- **[Resources →](resources/)**

---

**Last Updated**: February 2026
**Documentation Version**: 1.0
**Contract Version**: See individual contracts
