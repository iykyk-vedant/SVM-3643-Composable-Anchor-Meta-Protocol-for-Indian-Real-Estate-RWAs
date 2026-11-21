# Implementation Decisions

## 2026-11-21 — One Anchor program with three registry modules

**Decision:** P0 uses one deployable Anchor program. Its instructions and account types remain separated into `property`, `placement`, and `distribution` modules, with a shared property asset PDA.

**Reason:** This is a five-day prototype, and the checkout has no Rust/Solana/Anchor toolchain. One program avoids cross-program account deserialization, CPI, deployment and IDL coordination risks while still supporting explicit asset binding and three logically separated registries. The architecture docs explicitly permit this trade-off when cross-program overhead jeopardizes the deadline.

**Trade-offs:** A shared upgrade authority and deployment address reduce independent module upgradeability. Account types and instruction boundaries must stay modular so the programs can be split later. This does not change the three-registry product scope.

## Day-one checklist

- [x] Read `AGENTS.md`, `PRD.md`, and every file in `docs/`.
- [x] Inspect Git state, remote repository, and installed toolchain.
- [x] Choose one program with three separated modules.
- [x] Scaffold a pinned Anchor workspace and shared SDK/UI workspaces.
- [x] Implement issuer-owned asset identity and append-only document versions.
- [x] Add integration cases for unauthorized issuer, duplicate asset, and invalid version chain.
- [x] Run available SDK checks; record exact results below.

### Day-one milestone verification

- Passed: `pnpm exec ts-mocha -p tsconfig.json -t 100000 tests/sdk.ts` — 2 SDK tests.
- Passed: `pnpm exec tsc --noEmit --target ES2022 --module commonjs --moduleResolution node --strict --esModuleInterop --skipLibCheck sdk/src/index.ts sdk/src/property.ts`.
- Blocked: `cargo fmt --check`, `anchor build`, and Anchor integration tests — `cargo`, `anchor`, and `solana` are not installed; WSL has no Linux distribution. The property integration tests exist but are not counted as executed.

**Next task:** install/enable the documented Linux toolchain, compile and run the property integration tests, then complete verifier authorization and the Private Placement Registry per `docs/IMPLEMENTATION_PLAN.md`.

## Toolchain verification

The host has Node.js `v22.18.0` and npm `10.33.0`; `pnpm` is not available as a PowerShell command. `rustc`, `cargo`, `anchor`, and `solana` are not installed. `wsl.exe` exists but reports no installed Linux distribution. Solana's official installation guide says Windows development requires WSL. Consequently Anchor compilation and local-validator tests are currently blocked. Framework usage will be limited to documented Anchor account constraints and ordinary Rust APIs; no optional external protocol integrations are included.
