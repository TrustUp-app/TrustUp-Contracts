# Standards & Conventions

Code standards, conventions, and best practices for TrustUp smart contract development.

## Contents

- [Error Handling](error-handling.md) - Error codes, patterns, and best practices
- [File Organization](file-organization.md) - Repository and module structure
- [Code Style](code-style.md) - Rust code style and conventions
- [Naming Conventions](naming-conventions.md) - Naming standards for contracts, functions, variables

## Quick Reference

### Module Structure

Every contract follows this pattern:
```
src/
├── lib.rs        # Public API
├── types.rs      # Constants, DataKey
├── errors.rs     # Error enums
├── storage.rs    # Storage operations
├── access.rs     # Authorization
├── events.rs     # Event helpers
└── tests.rs      # Tests
```

### Naming Standards

- **Files**: `snake_case.rs`
- **Functions**: `snake_case()`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Contracts**: `kebab-case`

### Error Pattern

```rust
#[contracterror]
#[repr(u32)]
pub enum MyError {
    ErrorName = 1,  // Sequential from 1
}
```

### Testing Pattern

```rust
#[test]
fn test_name() {
    let env = Env::default();
    env.mock_all_auths();
    // ... test code
}
```

## Key Principles

1. **Consistency**: Follow established patterns
2. **Clarity**: Code should be self-documenting
3. **Safety**: Use checked operations, validate inputs
4. **Testing**: Comprehensive test coverage
5. **Documentation**: Document public APIs and complex logic

## See Also

- [Architecture](../architecture/) - System architecture
- [Development](../development/) - Development workflow
- [Resources](../resources/) - External tools and references
