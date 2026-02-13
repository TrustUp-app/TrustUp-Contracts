# Architecture Documentation

Technical architecture and design documentation for TrustUp smart contracts.

## Contents

- [Overview](overview.md) - System architecture, tech stack, contract interactions
- [Storage Patterns](storage-patterns.md) - Data storage strategies and patterns
- [Contract Design](contracts.md) - Individual contract architecture details

## Quick Reference

### Tech Stack
- **Blockchain**: Stellar
- **Platform**: Soroban (WASM)
- **Language**: Rust
- **SDK**: soroban-sdk 22.0.0

### Contract Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ Reputation  │◄────┤  CreditLine  │────►│   Merchant   │
│             │     │              │     │   Registry   │
└─────────────┘     └──────────────┘     └──────────────┘
      ▲                     │
      │                     ▼
      │             ┌──────────────┐
      └─────────────┤  Liquidity   │
                    │     Pool     │
                    └──────────────┘
```

### Key Principles

1. **Modularity**: Separate contracts for distinct concerns
2. **Security**: Auth-first, safe arithmetic, comprehensive testing
3. **Transparency**: Events for all state changes
4. **Efficiency**: Optimized WASM, minimal storage operations

## See Also

- [Standards](../standards/) - Code standards and conventions
- [Development](../development/) - Development workflow and tools
- [Resources](../resources/) - External references and tools
