# Security Invariants and Threat Model

## Invariants
S01: Every mutation requires the correct issuer, verifier or registrar signer.  
S02: All asset/offering/epoch PDAs derive from the provided canonical keys and verified owners.  
S03: Cross-program references cannot be substituted with unrelated accounts or fake program-owned data.  
S04: One unique offeree reference per offering; same person represented by one opaque ID, with off-chain identity mapping outside protocol.  
S05: State machine disallows premature, repeated and backwards transitions.  
S06: Closed offerings cannot accept new events.  
S07: Document version numbers are monotonic; predecessor and successor references cannot create cycles.  
S08: Unauthorized/revoked verifier cannot create new attestations; historical records preserve who attested at that time.  
S09: Distribution totals use checked arithmetic; no overflow/underflow.  
S10: Finalized epoch cannot be overwritten; corrections are separate linked records.  
S11: Merkle leaves bind asset, epoch, opaque investor ID, currency, nonce and amounts with versioned domain separation.  
S12: No raw PAN/Aadhaar/PII or confidential financial statements stored in public accounts/events.  
S13: A fake mint or unrelated asset cannot be attached to an existing record through account substitution.  
S14: Replayed instruction/receipt cannot double-register or modify finalized state.  
S15: Events and UI must never label attested claims as independently verified facts.

## Threats to test
- Unauthorized issuer impersonation.
- Fake verifier / registrar authority and stale revoked permissions.
- Duplicate offeree with same ID, multiple wallets and pseudonym collisions.
- Invalid offeree transitions and offering closure bypass.
- Maliciously large amounts and arithmetic overflow.
- Forged Merkle proof, wrong asset/epoch, ambiguous serialization.
- Document hash collisions in the **application protocol** caused by missing domain separation (not SHA-256 cryptanalytic collision).
- Sensitive information leaks via predictable identifiers, URIs, logs or screenshots.
- Indexer gaps: ensure persisted accounts, not events alone, carry critical state.

## Explicit non-guarantees
No property title validation, Indian regulatory certification, investor identity verification, tax calculation, bank transfer settlement or legally enforceable rights are provided by these programs. Do not write misleading comments, marketing or tests suggesting otherwise.
