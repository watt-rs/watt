## Hello 👋

Thanks you for considering contribution to this project. To make collaboration smooth and effective, please read the following rules, standards, and recommendations.

## Requirements 🧰

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (1.88)
- Git
- Cargo (comes with Rust)
- Rustup

Optional:
- Rust analyzer (recommended for IDE integration)

## Getting Started 📚

1. **Fork** this repository
2. **Clone** your fork to your local machine folder:
   ```bash
   git clone https://github.com/your-username/project-name.git
   ```
3. Try to build and run project.
  debug:
  ```bash
  cargo build
  ```
  release:
  ```bash
  cargo build --release
  ```
4. Contribute!

## Code style 🌡️

### Use consistent naming convetions.

Rust:
- snake_case for variables, functions and fields.
- PascalCase for struct and enum names.
 
Watt:
- snake_case for variable, functions, fields and units.
- PascalCase for types.

### Format

Keep code lines under 80 characters per row, follow the official Rust style rules:
https://doc.rust-lang.org/nightly/style-guide/

Use comments in your code, that will describe your code in detail, but don't overdo it

## Making changes 🔌

How to:
1. Create a new branch for your changes.
2. Commit your changes.
3. Push to your fork
4. Open a pull request on GitHub

## Before Submitting a Pull Request ✅

Please make sure:

- [ ] Your code builds without errors
- [ ] Your code is formatted (`cargo fmt`)
- [ ] You’ve written tests for new features or bugfixes
- [ ] Your branch is up to date with `main`
- [ ] Your branch haven't any conflicts with `main`

## Pull requests and commits format 📑

- Write commit names in the past tense. For example:
    ```
    fixed bug related to ...
    ```
    instead of
    ```
    fix bug
    ```
- Use clear, descriptive titles for PRs and commits
- Reference related issues using Fixes #issue_number or Closes #issue_number when applicable

## Code of Conduct 📐
Please, be respectful and open to feedback. Constructive discussion and diverse perspectives are always welcome.

## Need help? 🐞
Open an issue or contact the maintainer. We're happy to help!
