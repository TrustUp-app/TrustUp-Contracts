<img width="4554" height="1139" alt="TrustUp-Banner" src="https://github.com/user-attachments/assets/ee412e56-c481-49d6-879f-bde52f2b178a" />

<div align="center">

![Stellar](https://img.shields.io/badge/Stellar-7D00FF?style=for-the-badge&logo=stellar&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Soroban](https://img.shields.io/badge/Soroban-6B46C1?style=for-the-badge&logo=stellar&logoColor=white)
![WASM](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white)

[![Open Source](https://img.shields.io/badge/Open%20Source-Yes-green?style=flat-square)](https://opensource.org/)
[![Rust](https://img.shields.io/badge/Rust-1.75-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Soroban SDK](https://img.shields.io/badge/Soroban-22.0-purple?style=flat-square)](https://soroban.stellar.org/)

**Decentralized Buy Now Pay Later (BNPL) smart contracts on Stellar Network**

[Features](#-features) • [Tech Stack](#-tech-stack) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing)

</div>

---

## 📖 About

TrustUp Contracts is a suite of production-ready smart contracts powering decentralized Buy Now Pay Later (BNPL) on Stellar blockchain. Built with Rust and Soroban, it provides on-chain reputation, credit line management, merchant validation, and liquidity pool functionality.

### Key Features

- ⭐ **On-chain Reputation** - Immutable credit scores (0-100) with admin controls
- 💰 **Credit Line Management** - Loan creation, repayment, and default handling
- 🏪 **Merchant Registry** - Whitelist of authorized merchants
- 💧 **Liquidity Pool** - LP deposits, withdrawals, and interest distribution
- 🔐 **Access Control** - Role-based permissions (admin, updaters)
- 📊 **Event Emission** - Complete audit trail on-chain
- 🧪 **Battle-tested** - Comprehensive test coverage (37+ tests)
- 🔒 **Security First** - Safe arithmetic, input validation, OpenZeppelin patterns

## 🛠 Tech Stack

**RS1.75 · SSDK22 · WASM · OZ · CARGO**

### Core Technologies

| Category | Technology | Version |
|----------|-----------|---------|
| **Language** | Rust | 1.75+ |
| **SDK** | Soroban SDK | 22.0.0 |
| **Platform** | Stellar Soroban | Mainnet |
| **Build** | Cargo | Latest |
| **Target** | wasm32-unknown-unknown | - |
| **Security** | OpenZeppelin Stellar | Main |
| **Testing** | Soroban Testutils | 22.0.0 |

### Smart Contracts

- 🌟 **Stellar Network** - Layer 1 blockchain
- 🔷 **Soroban** - WASM smart contract platform
- 🦀 **Rust** - Memory-safe systems language
- 📦 **WASM** - Portable bytecode format

## 📁 Project Structure

```
TrustUp-Contracts/
├── contracts/
│   ├── reputation-contract/     # ✅ User credit scores (0-100)
│   ├── creditline-contract/     # ⏳ Loan management
│   ├── merchant-registry-contract/ # ⏳ Merchant whitelist
│   └── liquidity-pool-contract/ # ⏳ LP management
├── docs/
│   ├── architecture/            # System architecture
│   │   ├── overview.md          # Tech stack and design
│   │   ├── contracts.md         # Contract details
│   │   └── storage-patterns.md  # Storage strategies
│   ├── standards/               # Code standards
│   │   ├── error-handling.md    # Error patterns
│   │   ├── file-organization.md # Project structure
│   │   └── code-style.md        # Rust style guide
│   ├── development/             # Dev workflows
│   │   └── README.md            # Setup and tools
│   └── resources/               # External resources
│       ├── openzeppelin.md      # OpenZeppelin tools
│       ├── stellar-soroban.md   # Stellar docs
│       └── ai-assistants.md     # MCP servers
├── target/
│   └── wasm32-unknown-unknown/
│       └── release/*.wasm       # Deployable contracts
├── Cargo.toml                   # Workspace config
├── CONTRIBUTING.md              # Contribution guide
├── PROJECT_CONTEXT.md           # Project vision
└── README.md                    # This file
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75 or higher
- Cargo (included with Rust)
- wasm32-unknown-unknown target
- Stellar CLI (for deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/TrustUp-app/TrustUp-Contracts.git
cd TrustUp-Contracts

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI (optional, for deployment)
cargo install stellar-cli --locked
```

### Configuration

No configuration needed for development. Contracts are stateless and configured at deployment time.

For deployment configuration, see [Deployment Guide](./docs/deployment.md).

### Running the Application

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build all contracts (native)
cargo build --release

# Build WASM for deployment
cargo build -p reputation-contract --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/reputation_contract.wasm
```

### Contract Deployment

```bash
# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reputation_contract.wasm \
  --source alice \
  --network testnet

# Initialize contract
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize --admin <ADMIN_ADDRESS>
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p reputation-contract

# Run specific test
cargo test test_increase_score

# Run with output
cargo test -- --nocapture

# Check code coverage
cargo tarpaulin
```

## 📚 Documentation

Comprehensive documentation is available in the `docs/` folder:

- [Architecture Overview](./docs/architecture/overview.md) - System design and tech stack
- [Contract Details](./docs/architecture/contracts.md) - Individual contract specs
- [Storage Patterns](./docs/architecture/storage-patterns.md) - Data management
- [Error Handling](./docs/standards/error-handling.md) - Error codes and patterns
- [Code Style Guide](./docs/standards/code-style.md) - Rust conventions
- [File Organization](./docs/standards/file-organization.md) - Project structure
- [OpenZeppelin Tools](./docs/resources/openzeppelin.md) - Security libraries
- [Stellar & Soroban](./docs/resources/stellar-soroban.md) - Platform docs
- [AI Assistants & MCP](./docs/resources/ai-assistants.md) - Development tools
- [Contributing Guide](./CONTRIBUTING.md) - Development workflow
- [Project Context](./PROJECT_CONTEXT.md) - Vision and use cases

### Contract Documentation

Each contract includes inline documentation:

```bash
# Generate and view docs
cargo doc --open

# View specific contract docs
cargo doc -p reputation-contract --open
```

## 🏗 Architecture Principles

- **🔒 Security First** - Safe arithmetic, input validation, comprehensive tests
- **📊 Event-driven** - All state changes emit events for indexing
- **🧩 Modular** - Independent contracts with clear interfaces
- **⚡ Gas Optimized** - WASM size <64KB, minimal storage operations
- **✅ Battle-tested** - Extensive test coverage, OpenZeppelin patterns
- **🔗 Composable** - Contracts designed for integration

## 🔐 Security

- **Safe Arithmetic** - `checked_add/sub/mul/div` to prevent overflow
- **Input Validation** - All inputs validated before processing
- **Access Control** - Role-based permissions (admin, updaters)
- **Event Emission** - Complete audit trail
- **OpenZeppelin** - Industry-standard security patterns
- **Comprehensive Testing** - 37+ tests covering edge cases

### Security Checklist

- ✅ Authorization checks before state changes
- ✅ Safe arithmetic operations
- ✅ Input validation and range checks
- ✅ Event emission for all mutations
- ✅ Fail securely (panic on unexpected conditions)
- ⏳ External security audit (planned)

## 📦 Contracts Overview

### ✅ Reputation Contract (Complete)

Manages user credit scores (0-100) with role-based access control.

**Status**: Deployed to testnet
**Tests**: 37 passing
**Functions**: `get_score`, `increase_score`, `decrease_score`, `set_admin`, `set_updater`

### ⏳ CreditLine Contract (In Progress)

Handles loan creation, repayment, and default management.

**Status**: Partial implementation
**Progress**: 67% complete (2/3 issues)
**Remaining**: Loan repayment functionality

### ⏳ Merchant Registry (Planned)

Whitelist of authorized merchants.

**Status**: Not started
**Purpose**: Validate merchants before loan creation

### ⏳ Liquidity Pool (Planned)

Manages LP deposits, withdrawals, and interest distribution.

**Status**: Not started
**Purpose**: Fund loans and reward liquidity providers

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for:

- Development setup
- Code style guidelines
- Testing requirements
- Pull request process

### Quick Contribution Guide

1. **Pick an issue** from [Issues](https://github.com/TrustUp-app/TrustUp-Contracts/issues)
2. **Create branch**: `git checkout -b feat/SC-XX-description`
3. **Follow standards**: [Code Style](./docs/standards/code-style.md)
4. **Write tests**: Coverage goal >90%
5. **Run checks**: `cargo fmt && cargo clippy && cargo test`
6. **Submit PR**: Use the [PR template](./.github/PULL_REQUEST_TEMPLATE.md)

## 📊 Development Status

### Current Progress: 11/20 Issues Complete (55%)

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1: Access Control | ✅ Complete | 100% |
| Phase 2: Reputation | ✅ Complete | 100% |
| Phase 3: CreditLine Core | ⏳ Partial | 67% |
| Phase 4: Integration | ⏳ Partial | 0% |
| Phase 5: Merchant Registry | ⏳ Pending | 0% |
| Phase 6: Liquidity Pool | ⏳ Pending | 0% |
| Phase 7: Testing | ⏳ Partial | 33% |

See [ROADMAP.md](./docs/ROADMAP.md) for detailed breakdown.

## 🙏 Acknowledgments

- [Stellar Development Foundation](https://www.stellar.org/) - For the Soroban platform
- [OpenZeppelin](https://www.openzeppelin.com/) - For security standards and tools
- [Rust Community](https://www.rust-lang.org/community) - For the amazing language

## 📞 Support

- 📖 [Documentation](./docs/)
- 🐛 [Issue Tracker](https://github.com/TrustUp-app/TrustUp-Contracts/issues)
- 💬 [Discussions](https://github.com/TrustUp-app/TrustUp-Contracts/discussions)
- 💻 [Smart Contracts](https://github.com/TrustUp-app/TrustUp-Contracts)
- 🌐 [API Backend](https://github.com/TrustUp-app/TrustUp-API)

---
<!-- LEADERBOARD_START -->
## 🏆 Top 3 Contributors

<div align="center">

<table>
<tr>

<td align="center">
  <a href="https://github.com/KevinMB0220">
    <img src="https://avatars.githubusercontent.com/u/130603817?v=4" width="100px;" style="border-radius:50%;" alt="KevinMB0220"/><br />
    <sub><b>🥇 @KevinMB0220</b></sub><br />
    <sub>6 contributions</sub>
  </a>
</td>

<td align="center">
  <a href="https://github.com/ryzen-xp">
    <img src="https://avatars.githubusercontent.com/u/92181599?v=4" width="100px;" style="border-radius:50%;" alt="ryzen-xp"/><br />
    <sub><b>🥈 @ryzen-xp</b></sub><br />
    <sub>6 contributions</sub>
  </a>
</td>

<td align="center">
  <a href="https://github.com/davedumto">
    <img src="https://avatars.githubusercontent.com/u/112870446?v=4" width="100px;" style="border-radius:50%;" alt="davedumto"/><br />
    <sub><b>🥉 @davedumto</b></sub><br />
    <sub>6 contributions</sub>
  </a>
</td>

</tr>
</table>
</div>

<!-- LEADERBOARD_END -->

---
<div align="center">

**Built with ❤️ for the Stellar ecosystem**

[![Stellar](https://img.shields.io/badge/Powered%20by-Stellar-7D00FF?style=flat-square)](https://www.stellar.org/)
[![Soroban](https://img.shields.io/badge/Built%20with-Soroban-6B46C1?style=flat-square)](https://soroban.stellar.org/)
[![Open Source](https://img.shields.io/badge/Open%20Source-Yes-green?style=flat-square)](https://opensource.org/)

</div>
