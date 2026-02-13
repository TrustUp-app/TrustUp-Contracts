# OpenZeppelin Tools for Stellar/Soroban

OpenZeppelin provides battle-tested smart contract libraries and security tools for Stellar's Soroban platform.

## Overview

OpenZeppelin has partnered with Stellar (January 2025 - December 2026) to bring their security expertise to Soroban:
- 40 auditor weeks over two years
- Development of the Stellar Library with foundational contracts
- Advanced token standards and cryptographic utilities

## Libraries

### 1. OpenZeppelin Stellar Contracts

Security-audited contract templates and implementations in Rust.

**Repository**: https://github.com/OpenZeppelin/stellar-contracts

**Installation** (Cargo.toml):
```toml
[dependencies]
openzeppelin-stellar = { git = "https://github.com/OpenZeppelin/stellar-contracts", branch = "main" }
```

**Features**:
- Fungible token standards (SEP-41)
- Access control patterns
- Security best practices
- Battle-tested implementations

**Documentation**: https://docs.openzeppelin.com/stellar-contracts

### 2. Soroban Helpers

Rust libraries to simplify Soroban development and testing.

**Repository**: https://github.com/OpenZeppelin/soroban-helpers

**Installation** (Cargo.toml):
```toml
[dependencies]
openzeppelin-soroban-helpers = { git = "https://github.com/OpenZeppelin/soroban-helpers", branch = "main" }
```

**Features**:
- Testing utilities
- Common patterns and helpers
- Development tools

### 3. Security Detectors SDK

Framework for detecting vulnerabilities in Soroban contracts.

**Repository**: https://github.com/OpenZeppelin/soroban-security-detectors-sdk

**Use case**: Static analysis and security scanning

## Tools

### Contract Wizard

Interactive web-based contract generator.

**URL**: https://wizard.openzeppelin.com/stellar

**Features**:
- Generate secure Soroban contracts
- Select features and parameters
- Download ready-to-deploy code
- Based on OpenZeppelin templates

**Usage**:
1. Visit the wizard
2. Select contract type (Token, Access Control, etc.)
3. Configure parameters
4. Copy generated Rust code
5. Integrate into your project

### OpenZeppelin Wizard CLI

Generate contracts from command line.

**Installation**:
```bash
npm install @openzeppelin/wizard-stellar
```

**Usage**:
```bash
npx @openzeppelin/wizard-stellar
```

## Integration with TrustUp

### Current Setup

TrustUp uses OpenZeppelin libraries for:
- Security patterns
- Access control (admin, updaters)
- Safe arithmetic operations
- Contract templates

### Cargo Configuration

See [contracts/reputation-contract/Cargo.toml](/contracts/reputation-contract/Cargo.toml):
```toml
[dependencies]
openzeppelin-stellar = { git = "https://github.com/OpenZeppelin/stellar-contracts", branch = "main" }
openzeppelin-soroban-helpers = { git = "https://github.com/OpenZeppelin/soroban-helpers", branch = "main" }
```

### Recommended Patterns

1. **Access Control**: Use OpenZeppelin's access control patterns for admin/updater roles
2. **Token Standards**: Follow SEP-41 for any token implementations
3. **Security**: Apply OpenZeppelin security best practices
4. **Testing**: Use OpenZeppelin helpers for comprehensive tests

## Best Practices

### 1. Stay Updated
```bash
# Update dependencies regularly
cargo update
```

### 2. Use Wizard for Boilerplate
- Start new contracts with the wizard
- Customize generated code for specific needs
- Maintain OpenZeppelin patterns

### 3. Security First
- Follow OpenZeppelin security guidelines
- Use their audited implementations when available
- Run security detectors regularly

### 4. Read the Docs
- Review OpenZeppelin documentation
- Understand patterns before implementation
- Follow their recommended practices

## Resources

- **Main Site**: https://mcp.openzeppelin.com/
- **GitHub**: https://github.com/OpenZeppelin
- **Documentation**: https://docs.openzeppelin.com/
- **Stellar Integration**: https://developers.stellar.org/docs/tools/openzeppelin-contracts
- **Forum**: https://forum.openzeppelin.com/

## Common Commands

```bash
# Add OpenZeppelin to new contract
cargo add openzeppelin-stellar --git https://github.com/OpenZeppelin/stellar-contracts

# Check for updates
cargo update -p openzeppelin-stellar

# Build with OpenZeppelin dependencies
cargo build --release
```

## Support

- GitHub Issues: https://github.com/OpenZeppelin/stellar-contracts/issues
- Forum: https://forum.openzeppelin.com/
- Stellar Discord: #soroban channel
