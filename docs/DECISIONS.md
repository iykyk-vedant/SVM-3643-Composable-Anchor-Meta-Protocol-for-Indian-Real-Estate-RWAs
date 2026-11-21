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

**Follow-up:** the placement/distribution modules, SDK helpers, demo script, and UI were implemented in later milestones below. Anchor compilation and integration tests remain blocked on the missing Linux toolchain.

### Placement, distribution, and SDK milestone

- Implemented issuer-scoped registrar authorization, an offering PDA, unique offeree PDAs, and append-only offeree event accounts for registration/invitation/application/allotment/rejection/withdrawal.
- Implemented issuer-owned distribution epochs with checked arithmetic, finalization, and append-only sequential correction PDAs. Corrections remain linked to a finalized epoch and must preserve its currency and decimals.
- Added deterministic receipt serialization, SHA-256 leaves, sorted pair hashing, odd-node duplication, proof generation/verification, and `sdk/test-vectors/merkle.json`.
- Passed: `pnpm --filter @indian-rwa/registry-sdk typecheck`.
- Passed: `pnpm exec ts-mocha -p tsconfig.json -t 100000 tests/sdk.ts` — 5 tests, including tampered amount, wrong epoch, invalid totals, and the golden vector.
- Not run/blocked: `cargo fmt --check`, `cargo test`, `anchor build`, `anchor test`; no Rust/Solana/Anchor toolchain or local validator is installed.

**Next task:** add placement/distribution local-validator integration tests, then enable the Solana Linux toolchain and run formatter, Rust tests, Anchor build, integration tests, and the actual local lifecycle demo. Update remaining deployment/security limitations after results.

### SDK, fictional lifecycle demo, and UI milestone

- Added typed Anchor SDK transaction methods and fetch helpers for the three registry modules, plus canonical PDA derivation helpers.
- Added `sdk/src/demo.ts`, which submits synthetic asset, document, attestation, offering, offeree, and distribution transactions to a configured local validator; it checks valid and tampered receipt proofs.
- Added a Next.js App Router UI with Issuer, Registrar, Investor, and Auditor views. The Investor view performs a real local Merkle verification and tamper check. All other displayed UI data is clearly a fictional scenario preview; no wallet or transaction claims are made.
- Passed: SDK TypeScript typecheck and 5 SDK tests (hashing, canonical serialization, inclusion, tamper, wrong epoch, and fixed vector).
- Passed: `pnpm --filter @indian-rwa/demo-app build` using Next.js `16.4.6` and Node.js `22.18.0`.
- Not run/blocked: local lifecycle demo and Anchor integration suite; missing Rust/Anchor/Solana toolchain and validator.

**Next task:** add placement/distribution local-validator integration tests, then enable the Solana Linux toolchain and run formatter, Rust tests, Anchor build, integration tests, and the actual local lifecycle demo. Update remaining deployment/security limitations after results.

## Toolchain verification

The host has Node.js `v22.18.0` and npm `10.33.0`; `pnpm` is not available as a PowerShell command. `rustc`, `cargo`, `anchor`, and `solana` are not installed. `wsl.exe` exists but reports no installed Linux distribution. Solana's official installation guide says Windows development requires WSL. Consequently Anchor compilation and local-validator tests are currently blocked. Framework usage will be limited to documented Anchor account constraints and ordinary Rust APIs; no optional external protocol integrations are included.
