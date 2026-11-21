# Indian RWA Registry Protocol (fictional demo)

A five-day Solana prototype for three composable record registries: property documents, private placement administration, and rental distributions with receipt commitments. This repository is not a legal tokenization standard, title registry, KYC provider, tax engine, or compliance certification. All sample entities and values are fictional.

## Current implementation status

The workspace has issuer-owned asset PDAs, append-only document version PDAs, verifier authorization/attestation, a Private Placement Registry with registrar roles and persistent offeree event records, and a Rental Yield & Tax Receipt Registry with immutable epochs and append-only corrections. The SDK has typed transaction helpers, account fetchers, domain-separated hashing, and deterministic Merkle receipt generation/proof verification. A localnet lifecycle script and Next.js fictional scenario/receipt-proof UI are present. Rust/Anchor builds and validator-backed integration tests are verified. The UI is a local preview, not a transaction client.

## Requirements

- Windows: install WSL 2 and an Ubuntu distribution, then install Rust, Solana CLI and Anchor there. Solana's official guide states that Windows development uses WSL: [installation guide](https://solana.com/docs/intro/installation).
- Anchor CLI `0.31.1`, Solana CLI `2.2.1`, Rust stable supported by that Solana toolchain.
- Node.js `22.x` and pnpm `10.20.0`.

The workspace pins the Anchor and Solana CLI versions in `Anchor.toml` and Anchor Rust dependency in `Cargo.toml`. Verify the installed versions before building.

## Commands

```sh
pnpm install
cargo fmt --all -- --check
anchor build
anchor test
pnpm test:unit
pnpm --filter @indian-rwa/registry-sdk typecheck
pnpm --filter @indian-rwa/demo-app dev
```

For local development, configure a local Solana keypair and validator as described in the [Solana CLI docs](https://solana.com/docs/references/cli). `anchor test` starts the configured local validator, deploys the program, then runs the TypeScript integration tests.

To seed a local validator with synthetic records, run `solana-test-validator` in one terminal, then `anchor build`, `anchor deploy`, and `anchor run demo` in another. The demo creates an asset, versions and attests a fictional valuation commitment, records eight synthetic offeree references (six invited, four applied, three allotted), creates and finalizes one fictional distribution epoch, and checks a valid and tampered receipt proof. It requires an Anchor-generated IDL at `target/idl/indian_rwa_registry.json`.

## Property registry (implemented first slice)

- Asset PDA: `["asset", issuer, asset_id]`; only the signing issuer can create its asset account.
- Document PDA: `["doc", asset, document_key, version_le_u64]`.
- Version 1 has no predecessor. Later versions require the immediately preceding record for the same asset and opaque document key. Each version has a distinct immutable account.
- Hash inputs should use random opaque IDs and off-chain controlled documents. A hash proves byte integrity only; it does not establish authenticity, title or legal effect.

## Security and limitations

- No raw identity data, tax IDs, private investor data, confidential documents or live payment information belong in public accounts or events.
- Issuer and verifier statements are attributable claims, not independent legal findings. The protocol does not verify property title, regulatory eligibility, tax treatment, payment settlement or tax remittance.
- The planned offeree registry counts opaque people references, not wallet holders; its configurable administrative limit will not encode a universal legal cap.
- All legal and tax decisions remain off-chain and require appropriate professional review. No real asset, investor, KYC, banking or money is used.
- One program contains three separated modules to avoid cross-program integration overhead for this prototype; module account and instruction boundaries are intended to remain separable.

See `docs/SECURITY.md`, `docs/ARCHITECTURE.md`, `docs/IMPLEMENTATION_PLAN.md`, and `docs/DECISIONS.md` for full invariants, plans, and verification status.
