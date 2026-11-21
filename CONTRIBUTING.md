# Contributing

Contributions are welcome. For substantial changes, open an issue first to discuss the proposed behavior or interface.

## Development checks

Install the pinned JavaScript dependencies with `pnpm install --frozen-lockfile`. For Rust and Solana work, use the toolchain versions listed in the README.

Before opening a pull request, run the relevant checks:

```sh
cargo fmt --all -- --check
cargo test --workspace
anchor build
pnpm test
pnpm test:unit
pnpm typecheck
pnpm --filter @indian-rwa/demo-app build
```

Keep test fixtures fictional. Do not add real personal, financial, title, or confidential data. Preserve append-only records and document the security or compatibility impact of protocol changes.
