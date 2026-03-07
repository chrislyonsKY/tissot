# Contributing to Tissot

Thanks for helping build Tissot.

## Development setup

1. Install Rust stable (1.83+).
2. Clone the repository.
3. Run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
```

## Branch and PR flow

1. Create a branch from `main`.
2. Keep changes focused.
3. Add or update tests for behavior changes.
4. Open a pull request with:
   - problem statement
   - implementation summary
   - validation evidence (commands + output snippets)

## Coding expectations

- No `unwrap()` or `expect()` in library code.
- Use `thiserror` for library errors; `anyhow` is CLI-only.
- Use `geo` primitives for geometry operations.
- Use `proj` for CRS transforms.
- Keep visual-first behavior as the default UX.

## Commit style

Use concise, imperative commit messages, for example:

- `fix: handle null geometry in duplicate checker`
- `docs: add architecture diagram`
- `ci: enforce clippy warning-free build`

## Reporting bugs and requesting features

Please use the issue templates in `.github/ISSUE_TEMPLATE/`.
