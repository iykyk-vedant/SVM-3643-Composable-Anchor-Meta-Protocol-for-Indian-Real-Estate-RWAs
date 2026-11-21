# External Dependencies and Verification Ledger

## Principle
No external protocol integration is required to demonstrate the three registries. Do not assume MPL-3643 is available on localnet/devnet, audited, or compatible with a particular SDK. Verify official repos/docs and actual deployment before adding it.

## Dependencies to verify before coding
| Dependency | Purpose | Verification status |
|---|---|---|
| Rust / Solana / Anchor | Programs, deployment, testing | UNVERIFIED in this workspace |
| `anchor-lang` and Anchor TypeScript client | Account validation and SDK | UNVERIFIED in this workspace |
| Node + pnpm | SDK and UI | UNVERIFIED in this workspace |
| SHA-256 library | Document hashes / Merkle proofs | Choose and verify against test vectors |
| Next.js | Optional reference app | UNVERIFIED in this workspace |
| Token-2022 | Optional informational asset mint link | Not required for P0 |
| SAS | Optional identity attestation integration | Not required for P0 |
| MPL-3643 / Token ACL | Optional future transfer restrictions | Not required for P0 |

## Official starting points (check URLs and APIs before use)
- Solana documentation: https://solana.com/docs
- Anchor documentation: https://www.anchor-lang.com/docs
- SPL Token-2022 overview: https://solana.com/docs/tokens/extensions
- Metaplex docs: https://www.metaplex.com/docs

## Legal validation backlog (not software assertions)
- What is the legal instrument: SPV equity, debt, REIT/SM REIT units, contractual rights or another security?
- Which issuer, investor, offering and distribution rules actually apply?
- Which documents can be public, privately shared, or only represented by commitments?
- What tax withholding provisions apply to each investor and distribution type?
- What issuer, registrar, custodian and trustee permissions are needed?

These questions must be resolved by appropriately qualified Indian professionals before any real-world use. The hackathon demo uses simulated data only.
