# Architecture and Interface Contracts

## System boundaries
**On-chain (trusted for state transitions):** Anchor programs, authority checks, hashes, immutable records, counters, receipt root.  
**Off-chain (trusted claims, NOT verified by chain):** legal identity mapping, source documents, document authenticity, title status, eligibility, rental accounting, tax treatment, bank transfers, payment confirmations.

## Program topology
Preferred three Anchor programs:
1. `property_registry` owns AssetRegistry, VerifierAuthorization and DocumentVersion.
2. `placement_registry` owns Offering and Offeree state, validates property-registry asset ownership/identity through a verified account and/or CPI.
3. `distribution_registry` owns DistributionEpoch and CorrectionEpoch, validates asset link.

If cross-program account decoding/IDL/CPI overhead jeopardizes the five-day deadline, use **one Anchor program with three clearly separated instruction modules** and document the choice. Never skip cross-module asset relationship checks.

## PDA plan (proposed, verify lengths)
- asset: `[b"asset", issuer_pubkey, asset_id_32]`
- verifier: `[b"verifier", asset_pubkey, verifier_pubkey]`
- document version: `[b"doc", asset_pubkey, document_key_32, version_le_u64]`
- offering: `[b"offering", asset_pubkey, offering_id_32]`
- offeree: `[b"offeree", offering_pubkey, offeree_ref_32]`
- distribution: `[b"epoch", asset_pubkey, epoch_id_le_u64]`
- correction: `[b"correction", epoch_pubkey, correction_index_le_u32]`

## Authority model
- Issuer controls asset and issuer-only actions.
- Issuer can authorize/revoke verifier and registrar. Distinguish current authorization from historical validity.
- Verifier signs own attestation and must be authorized for specific asset.
- Registrar can write offering events only with explicit delegated authority.
- Investor needs no private key to verify a Merkle proof; off-chain delivery of their proof is access controlled.

## Data conventions
- All identifiers are opaque 32-byte values, generated from high-entropy input.
- Amounts are `u64` in explicitly stated smallest currency units; arithmetic uses checked operations / `u128` intermediate when needed.
- `currency_code` is fixed-format (e.g. INR or USDC label) with explicit decimals. A bank transfer is not a token transfer.
- All hashes are SHA-256 over versioned, domain-separated canonical encodings. Publish exact encoding and golden vectors.
- Events are indexable hints; authoritative state lives in accounts.
- Account size must include Anchor discriminator, bounded vectors and future version fields. Avoid unbounded `String`/`Vec`.
- Include version, bump, creation timestamp, issuer authority and stable relationship keys.

## Important trust caveats
- Solana cannot validate a title deed or land registry independently.
- Section 42 offering obligations are not implemented by a `max_holders` or `max_offerees` counter.
- The tax registry records supplied figures; it cannot establish the correctness of tax calculations or remittance.
- A receipt proof only proves inclusion in a committed Merkle root.
- Mint linkage is informational in P0; no transfer restrictions are enforced by these registries.
