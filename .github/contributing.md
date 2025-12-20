# Contributing Guidelines

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Code Contributions](#code-contributions)
- [Development Setup](#development-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Pull Request Process](#pull-request-process)
- [Questions?](#questions)

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
   ```bash
   git clone https://github.com/YOUR_USERNAME/kanban_be.git
   cd kanban_be
   ```
3. Set up the development environment (see [Development Setup](#development-setup))
4. Create a feature branch (follow [Conventional Branches](https://conventional-branch.github.io/))
   ```bash
   git checkout -b feat/your-feature-name
   ```

## How to Contribute

### Reporting Bugs

Found a bug? Please use our [Bug Report template](https://github.com/mysci4k/kanban_be/issues/new?template=bug_report.yml) which will guide you through providing:

- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, database versions)
- Relevant logs or error messages

The template ensures all necessary information is included for quick resolution.

### Suggesting Features

Have an idea for improvement? Use our [Feature Request template](https://github.com/mysci4k/kanban_be/issues/new?template=feature_request.yml) to propose:

- Problem statement and motivation
- Proposed solution
- Alternative approaches considered
- Implementation details (if you have ideas)
- Use case examples

The template helps structure your request and estimate complexity.

**Before submitting:**
- Check [existing issues](https://github.com/mysci4k/kanban_be/issues) to avoid duplicates
- For major features, consider opening a [Discussion](https://github.com/mysci4k/kanban_be/discussions) first

### Documentation Improvements

Found unclear or missing documentation? Use our [Documentation template](https://github.com/mysci4k/kanban_be/issues/new?template=documentation.yml) to suggest improvements.

### Questions

Need help? You have several options:
- Use the [Question template](https://github.com/mysci4k/kanban_be/issues/new?template=question.yml) for specific questions
- Start a [Discussion](https://github.com/mysci4k/kanban_be/discussions) for broader topics
- Check existing [issues](https://github.com/mysci4k/kanban_be/issues) and [discussions](https://github.com/mysci4k/kanban_be/discussions)

### Code Contributions

We welcome code contributions! Before starting work on a significant change:

1. Check existing issues to avoid duplicate work
2. Open an issue to discuss your proposed changes
3. Wait for feedback before implementing large features

## Development Setup

### Prerequisites

- **Rust** 1.90+ (edition 2024)
- **PostgreSQL** 18+
- **Redis** 8+
- **SMTP Server**
- **Docker & Docker Compose** (optional, for local databases)

### Initial Setup

1. **Install Rust toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```

2. **Install development tools** (optional but recommended)
   ```bash
   cargo install --locked bacon
   ```

3. **Set up environment variables**
   ```bash
   cp .env.example .env
   ```

4. **Start databases**
   ```bash
   docker-compose up -d
   ```

5. **Run migrations**
   ```bash
   cargo run --package migration up
   ```

6. **Run the application**
   ```bash
   cargo run

   # Alternatively, run with auto-reload on file changes
   bacon run-long
   ```

### Development Workflow

```bash
# Auto-reload on file changes
bacon run-long

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Build release version
cargo build --release
```

## Code Style Guidelines

### Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Ensure `cargo clippy` passes with no warnings
- Write documentation comments for public APIs
- Use meaningful variable and function names
- Keep functions focused and testable

### Naming Conventions

```rust
// Modules: snake_case
mod user_repository;

// Types: PascalCase
struct UserDto { }
enum Role { }
trait UserRepository { }

// Functions and variables: snake_case
fn create_user() { }
let user_id = Uuid::now_v7();

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRY_ATTEMPTS: u32 = 3;

// Lifetimes: short, lowercase
fn example<'a>(data: &'a str) -> &'a str { }
```

### Error Handling

```rust
// Use Result for operations that can fail
async fn find_by_id(&self, user_id: Uuid) -> Result<User, ApplicationError> {
    // Implementation
}

// Use ? operator for error propagation
let user = self.user_repository.find_by_id(id).await?;

// Provide context with custom errors
return Err(ApplicationError::NotFound {
    message: "User not found".to_string(),
});
```

### Async/Await

```rust
// Use async/await for I/O operations
async fn create_user(&self, dto: CreateUserDto) -> Result<UserDto, ApplicationError> {
    let user = self.user_repository.create(dto).await?;
    Ok(UserDto::from_domain(user))
}

// Avoid blocking operations in async functions
// Use tokio::task::spawn_blocking for CPU-intensive work
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `style`: Code style changes (formatting, etc.)

**Examples:**
```bash
feat: add password reset functionality
fix: correct member permission check
docs: update installation instructions
refactor: simplify position calculation
test: add integration tests for registration
chore: update actix-web to 4.11.0
```

## Pull Request Process

1. **Update your branch with the latest main**
   ```bash
   git fetch origin
   git rebase origin/main
   ```

2. **Run all checks**
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```

3. **Commit your changes** (with conventional commit messages)
   ```bash
   git add .
   git commit -m "feat: add password reset functionality"
   ```

4. **Push to your fork**
   ```bash
   git push origin feat/your-feature-name
   ```

5. **Create Pull Request on GitHub**
   
   Our [PR template](.github/pull_request_template.md) will guide you through providing:
   - Description of changes
   - Type of change (fix, feature, breaking change, etc.)
   - Related issues
   - List of changes made
   
   **Important:** Add the appropriate prefix to your PR title:
   - `[FIX]` - Bug fixes
   - `[FEAT]` - New features
   - `[BREAKING]` - Breaking changes
   - `[REFACTOR]` - Code refactoring
   - `[DOCS]` - Documentation
   - `[TEST]` - Tests
   - `[CHORE]` - Maintenance
   - `[PERF]` - Performance improvements

### Review Process

- At least one approval required
- All CI checks must pass
- Address review comments promptly
- Maintainers may request changes or provide feedback

## Questions?

If you have questions or need help:

- 💬 Open a [Discussion](https://github.com/mysci4k/kanban_be/discussions)
- 📝 Comment on existing issues
- 📧 Contact the maintainers

---

**Thank you for contributing! 🎉**
