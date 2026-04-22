# Contributing to FOS神经元控制器

Thank you for your interest in contributing to **FOS神经元控制器** (FOS Neural Controller)!

This document provides guidelines for contributing to the FOS project.

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. We are committed to making participation in this project a harassment-free experience for everyone.

Expected behavior:
- Use welcoming and inclusive language
- Be respectful of differing viewpoints
- Accept constructive criticism gracefully
- Focus on what is best for the community

Unacceptable behavior:
- Harassment of any kind
- Personal or political attacks
- Publishing others' private information
- Other unethical practices

---

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include reproduction steps
4. Attach relevant logs/screenshots

### Suggesting Features

1. Check existing feature requests
2. Use the feature request template
3. Explain the use case and expected behavior
4. Discuss implementation approach

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes following the code style
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --workspace`
6. Update documentation if needed
7. Submit a pull request

---

## Development Setup

### Prerequisites

- Rust 1.76+
- PostgreSQL 15+ (for integration tests)
- Redis 7.0+ (optional)

### Building

```bash
# Clone the repository
git clone https://github.com/fos-platform/fos.git
cd fos

# Build the project
cargo build --release

# Run tests
cargo test --workspace
```

### Code Style

We follow standard Rust idioms:

- Use `cargo fmt` for code formatting
- Use `cargo clippy` for linting
- Enable `#[deny(warnings)]` in lib.rs

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

---

## Module Structure

### Adding a New Module

1. Create directory: `src/new-module/`
2. Add `Cargo.toml`:

```toml
[package]
name = "fos-new-module"
version = "0.1.0"
edition = "2021"

[dependencies]
fos-common.workspace = true
# ... other dependencies

[lib]
name = "fos_new_module"
path = "src/lib.rs"
```

3. Create `src/lib.rs`:

```rust
//! New Module Description

pub mod submodule;

pub use submodule::YourType;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        // Test implementation
    }
}
```

4. Add to workspace in `src/Cargo.toml`:

```toml
members = [
    "common",
    # ... existing modules
    "new-module",
]
```

5. Add binary entry point (optional):

```toml
[[bin]]
name = "fos-new-module"
path = "src/main.rs"
```

---

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific module
cargo test -p fos-gateway

# Run with output
cargo test --workspace -- --nocapture

# Run with coverage
cargo tarpaulin --workspace
```

### Writing Tests

Follow these patterns:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = your_function(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async code
    }
}
```

---

## Documentation

### Module Documentation

Document each module with:

```rust
//! # Module Name - Description
//!
//! ## Core Responsibilities
//! - Feature 1
//! - Feature 2
//!
//! ## Safety Rules
//! - Rule 1
//! - Rule 2
```

### API Documentation

- Document public types and functions
- Include usage examples
- Update README.md if adding major features

---

## Commit Messages

Follow conventional commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Code style
- `refactor`: Refactoring
- `test`: Testing
- `chore`: Maintenance

Example:
```
feat(lock): add reentrant lock support

Add reentrant lock that allows the same thread to acquire
the lock multiple times without blocking.

Fixes #123
```

---

## Review Process

1. **CI Checks**: All tests must pass
2. **Code Review**: At least one approval required
3. **Documentation**: Update if needed
4. **Merging**: Squash and merge to main

---

## Security Considerations

### Security Rules

FOS神经元控制器遵循四条安全铁律：

1. **信号链唯一**: 所有信号必须通过感觉→脊髓→运动三层
2. **脊髓不可绕过**: 脊髓校验不可跳过
3. **反射弧隔离**: 危险动作反射执行，不上传大脑
4. **反馈必闭环**: 执行结果必须反馈到控制层

其他安全实践：
- 永远不要提交密钥或凭证
- 使用环境变量处理敏感数据
- 确保所有设备控制都通过MCP层

### Reporting Security Issues

If you find a security vulnerability, please email: security@fos-platform.org

---

## License

By contributing to FOS, you agree that your contributions will be licensed under the Apache-2.0 License.

---

## Contact

- GitHub Issues: https://github.com/fos-platform/fos/issues
- Email: fos-team@example.com
- Website: https://fos-platform.github.io/docs

---

Thank you for contributing to FOS!