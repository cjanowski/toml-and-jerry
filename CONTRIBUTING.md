# Contributing to TOML and Jerry

Thank you for your interest in contributing to TOML and Jerry! This document provides guidelines and instructions for contributing to this project.

## Overview

TOML and Jerry is a tool for validating JSON/TOML/YAML/HCL configuration files against JSON Schema or OpenAPI component schemas. It can also generate starter JSON Schemas from Rust types.

## Development Setup

1. Ensure you have Rust installed (latest stable version recommended)
   ```bash
   rustup update stable
   ```

2. Clone the repository
   ```bash
   git clone https://github.com/cjanowski/toml-and-jerry.git
   cd toml-and-jerry
   ```

3. Install dependencies and build the project
   ```bash
   cargo build
   ```

4. Run tests
   ```bash
   cargo test
   ```

## Parallel Processing

This project uses [Rayon](https://docs.rs/rayon/latest/rayon/) for parallel processing. When implementing new features that involve processing multiple files or large datasets, consider using Rayon's parallel iterators:

```rust
use rayon::prelude::*;

// Example: Parallel processing of multiple files
files.par_iter().for_each(|file| {
    // Process each file in parallel
});
```

Key Rayon features used:
- `par_iter()` for parallel iteration
- `par_extend()` for parallel collection building
- `join()` for parallel task execution
- `scope()` for structured parallel tasks

When using Rayon, ensure that:
1. The work being parallelized is CPU-intensive enough to justify the overhead
2. The parallel code is thread-safe and doesn't introduce race conditions
3. The parallel implementation is tested with various input sizes
4. Consider using `rayon::ThreadPoolBuilder` for custom thread pool configuration if needed

## Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `rustfmt` to format your code
  ```bash
  cargo fmt
  ```
- Run `clippy` to check for common issues
  ```bash
  cargo clippy
  ```

## Making Changes

1. Create a new branch for your feature or bugfix
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following the code style guidelines

3. Add tests for new functionality

4. Run the test suite to ensure everything passes
   ```bash
   cargo test
   ```

5. Commit your changes with clear, descriptive commit messages

6. Push your branch and create a pull request

## Pull Request Process

1. Update the README.md with details of changes if needed
2. Update the documentation if you're adding new features
3. Ensure all tests pass
4. The PR will be reviewed by maintainers

## Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Add integration tests for new features
- Test with different input formats (JSON, TOML, YAML, HCL)

## Documentation

- Document new features and changes
- Update existing documentation if needed
- Include examples for new functionality
- Add comments to complex code sections

## Questions or Issues?

If you have any questions or run into issues, please:
1. Check existing issues to see if your question has already been answered
2. Create a new issue if needed

## License

By contributing to TOML and Jerry, you agree that your contributions will be licensed under the project's license. 
