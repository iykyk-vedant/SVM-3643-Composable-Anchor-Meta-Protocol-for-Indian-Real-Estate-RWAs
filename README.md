# Indian RWA Registry Protocol

An experimental Solana program for recording property-document commitments, private-placement administration events, and distribution receipt commitments. The repository includes a TypeScript SDK, validator tests, a fictional lifecycle demo, and a small Next.js interface.

This is a prototype. It does not establish property title, regulatory compliance, tax treatment, payment settlement, or tax remittance. All demo data is fictional.

## Registries

- **Property documents:** issuer-owned asset records, append-only document versions, verifier authorization and attestations.
- **Private placement:** offerings, opaque offeree references, registrar permissions, and append-only event history.
- **Rental distribution:** immutable distribution epochs, arithmetic checks, receipt Merkle roots, and append-only corrections.

The registries are separate modules in one Anchor program. The SDK provides transaction helpers, account fetchers, document hashing, and receipt-proof generation and verification.

## Requirements

- Rust and Cargo
- Solana CLI 2.2.1 and Anchor CLI 0.31.1
- Node.js 22 and pnpm 10.20.0
- On Windows, use WSL 2 with Ubuntu for the Solana toolchain

## Build and test

```sh
pnpm install --frozen-lockfile
anchor build
anchor keys sync
anchor build
anchor test
pnpm test:unit
pnpm typecheck
pnpm --filter @indian-rwa/demo-app build
```

On a fresh checkout, the first build creates the local program keypair. `anchor keys sync` aligns the declared program address with that keypair; the second build produces the matching artifact. `anchor test` starts a local validator and runs the integration suite.

## Run the fictional demo

After building and syncing the program, start a local validator with the program loaded at genesis:

```sh
solana-test-validator --reset \
  --bpf-program target/deploy/indian_rwa_registry-keypair.json \
  target/deploy/indian_rwa_registry.so
```

In another terminal, run:

```sh
anchor run demo
```

The demo records a fictional property, two document versions, eight opaque offeree references, and one distribution epoch. It prints funnel counts and checks a valid and tampered receipt proof. The proof demonstrates inclusion in a committed root; it does not demonstrate payment.

To run the UI locally:

```sh
pnpm --filter @indian-rwa/demo-app dev
```

## Security and limitations

- Do not put personal data, identity documents, tax IDs, private contracts, or live payment information on-chain.
- Hashes show byte integrity, not document authenticity or legal title. Issuer and verifier records remain attributable claims.
- Offeree counts are not wallet-holder counts and do not implement a statutory offering limit.
- Distribution records do not calculate taxes or prove bank settlement or tax remittance. Legal and tax decisions remain off-chain.
- This prototype is not intended for production assets or real funds.

See [Architecture](docs/ARCHITECTURE.md), [Security](SECURITY.md), [Decisions](docs/DECISIONS.md), and [External dependencies](docs/EXTERNAL_DEPENDENCIES.md).

## License

Licensed under the [Apache License 2.0](LICENSE).
