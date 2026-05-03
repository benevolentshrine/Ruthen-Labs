# Contributing Guidelines

Welcome to SumiLabs. This guide outlines the strict rules for the 3 founders (Lichi, Sayan, Kshitij) to maintain high-quality code and ensure operational discipline.

## 1. DCO Enforcement
Every commit **MUST** be signed to indicate your agreement to the Developer Certificate of Origin (DCO).
- Use `git commit -s` or `git commit --signoff`.
- To fix an unsigned commit, use `git commit --amend --no-edit --signoff` and force push if necessary.

## 2. Daily Push Rule
- Every founder is required to push **at least one commit daily**. Consistency is key.

## 3. Pull Request Process
- **No direct pushes to `main`**. All changes must go through a Pull Request.
- Every PR requires **at least 1 approval** from another founder before merging.
- All code must be reviewed and tested. Use the provided PR template.

## 4. Continuous Integration (CI) Requirement
- Merges to `main` are **strictly blocked** if the CI pipeline fails. 
- Ensure your code passes all tests and linting before requesting a review.

## 5. Code Style
- **Rust:** All Rust code must be formatted using `cargo fmt` and pass `cargo clippy`.
- **Go:** All Go code must be formatted using `gofmt` and pass `go vet`.

By strictly following these rules, we ensure the integrity, security, and quality of SumiLabs.
