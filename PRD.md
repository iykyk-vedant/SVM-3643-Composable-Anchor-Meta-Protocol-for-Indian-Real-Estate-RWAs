# PRD — Indian RWA Registry Protocol (Hackathon MVP)

**Status:** implementation specification / not legal advice  
**Target:** five-day Solana hackathon  
**Technology:** Rust, Anchor, Solana, TypeScript, Next.js (minimal UI)  
**Product:** one open-source, composable protocol with three registry modules and a shared SDK.

## 1. Problem and purpose
Generic token infrastructure can represent and restrict tokens, but an Indian real-estate issuer also needs a consistent, verifiable history of (a) evidence about the asset, (b) an offering's administrative events, and (c) distributions and investor receipts. Today these records are often off-chain. This prototype tests whether publishing authenticated commitments and append-only events on Solana makes the lifecycle more auditable and reusable across issuers.

This is a **record/evidence protocol**, not a legal tokenization standard, property-title registry, tax engine, bank, KYC provider, or certification of Indian regulatory compliance. Commercial demand and novelty are unverified; this MVP is a technical demonstration.

## 2. Users
- **Issuer/SPV operator:** creates asset, offerings and distribution epochs; signs issuer statements.
- **Authorized document verifier:** attests to document review, with explicit scope and revocation/supersession record.
- **Registrar/placement operator:** records identified-offeree and allotment events; synthetic identities for demo.
- **Investor:** sees public asset and offering summaries and verifies their private receipt using an off-chain proof.
- **Auditor:** inspects event history and verifies hash/proof integrity; cannot infer legal truth from hashes alone.

## 3. Scope
### P0 — mandatory
1. **Property Document Registry:** create asset; bind immutable issuer and optional token mint; register document commitments; verifier approval; versioning/supersession; query history.
2. **Private Placement Registry:** create offering; register unique pseudonymous offeree references; record invitation, application and allotment lifecycle; auditable state transitions; per-offering counters; operator roles; configurable administrative limit with explicit non-legal semantics.
3. **Rental Yield & Tax Receipt Registry:** create distribution epoch; record currency/decimals, gross/distributable/withheld/net totals, record-date snapshot reference, receipt Merkle root and evidence commitments; verify a private investor receipt/proof off-chain; append correction epoch referencing prior epoch.
4. **Shared TypeScript SDK:** typed methods for all three modules; local document hashing; receipt canonicalization, Merkle tree/proof creation and verification; transaction builders; account fetchers; clear errors.
5. **Demo:** one synthetic Mumbai commercial property, one offering, 5–10 fictional offerees, 3–5 allotments, one fictional monthly distribution, document update and proof verification.
6. **Tests:** successful paths plus unauthorized writes, duplicate offerees, invalid state transitions, invalid verifiers, invalid arithmetic, forged receipts, mismatched mint/asset and replay attempts.

### P1 — if P0 complete
- On-chain receipt claim marking / payout reconciliation (without pretending bank settlement is on-chain).
- Better multi-asset dashboard and role delegation/revocation.
- Exportable auditor reports and structured evidence manifests.

### Out of scope
- Actual Indian property token issuance or sale; title ownership transfer; real KYC; government or RERA API integration; bank/INR settlement; real withholding/tax filing; SEBI/IFSCA licensing; legal validity certification; automated statutory compliance; direct integration with MPL-3643 or Token ACL unless verified and time permits.

## 4. Shared design
Every registry references a single `AssetRegistry` PDA with an issuer and `asset_id` (32-byte unique ID), optional Token-2022 mint and schema version. Do not equate a token to legal ownership. Prefer `asset` PDA derived from `[b"asset", issuer, asset_id]` and program-specific PDAs scoped to asset/offering/epoch.

All mutations require appropriate signers and enforce cross-account relationships. Every record contains `created_at`, authority and immutable identity. Updates create new version records, not replacement of old document or financial evidence. Include program events for indexers, but do not rely on logs as the only persistent source of truth.

### 4A. Property Document Registry
**Core entities:** AssetRegistry, VerifierAuthorization, DocumentVersion.

**Instructions:**
- `create_asset(asset_id, metadata_commitment, optional_mint)` issuer-signed; one asset per ID.
- `authorize_verifier(verifier, scope)` / `revoke_verifier` issuer-signed; scope bound to asset.
- `add_document_version(document_key, version, document_hash, manifest_hash, previous_version)` issuer-signed; enforce monotonic versions and predecessor.
- `attest_document(document_version, review_commitment, outcome)` verifier-signed; verifier must be active and authorized for that asset/type.
- `supersede_document(document_version, successor)` issuer-signed; append explicit supersession reference, no deletion.

**Document types:** title-search report, valuation, SPV evidence, applicable RERA records, lease/rent statement, other. Type enum is an administrative label, not a legal finding.

**Acceptance:** any observer can check issuer signature, verifier authorization at time of attestation, hashes, and document version chain. Demo modifies a valuation report and shows old/new hashes.

**Privacy:** public hashes and timestamps can still leak information if low-entropy data is hashed; use opaque random document IDs and access-controlled off-chain originals. No public personal document links by default.

### 4B. Private Placement Registry
**Core entities:** Offering, Offeree, OfferingEvent or explicit per-person state record.

**Instructions:**
- `create_offering(offering_id, asset, fiscal_period_label, terms_commitment, optional_administrative_offeree_limit)` issuer/authorized registrar-signed.
- `register_offeree(offeree_ref)` registrar-signed, unique per offering; offeree_ref must be an opaque random 32-byte reference from an off-chain verified person mapping.
- `record_invitation(offeree, invitation_commitment)` registrar-signed, only from registered.
- `record_application(offeree, application_commitment)` registrar-signed, only after invitation.
- `record_allotment(offeree, amount, allotment_commitment)` registrar-signed, only after application; prevent repeated allotment unless explicitly designed as separate append-only events.
- `reject_or_withdraw(offeree, reason_commitment)` registrar-signed, only valid states.
- `close_offering` issuer-signed, no new events afterward.

**State machine:** REGISTERED -> INVITED -> APPLIED -> ALLOTTED; APPLIED -> REJECTED; REGISTERED/INVITED/APPLIED -> WITHDRAWN where documented. No backwards transitions. If real business process requires amendments, use separate corrective events.

**Counters:** number of unique registered offerees, invitations, applications, allottees. Track **people**, not wallet accounts. Configurable cap is an issuer-defined administrative guard, **not** an implementation of Companies Act Section 42. Do not assume a universal 200-person rule or a fixed financial-year treatment; exclusions and aggregate rules require legal review.

**Acceptance:** duplicate registration rejected, repeated invitation rejected, allotment before application rejected, unapproved operator rejected, close prevents new actions. Demonstrate a single person linked to multiple demo wallets off-chain but counted once.

### 4C. Rental Yield & Tax Receipt Registry
**Core entities:** DistributionEpoch, optional CorrectionEpoch; individual investor records remain off-chain.

**Instructions:**
- `create_distribution_epoch(asset, epoch_id, currency_code, decimals, record_date, snapshot_commitment, gross, expenses_and_reserves, distributable, total_withheld, total_net, merkle_root, evidence_manifest_commitment)` issuer-signed.
- `finalize_distribution_epoch` issuer-signed after review; finalized records immutable.
- `record_correction(previous_epoch, corrected_merkle_root, revised_totals, reason_commitment)` issuer-signed, append-only, explicit supersession chain; corrections must not erase original.

**Arithmetic invariant (for demo):** gross >= expenses_and_reserves; distributable = gross - expenses_and_reserves; total_net = distributable - total_withheld; withheld <= distributable. Document that actual accounting can involve other adjustments and should be configured off-chain in production.

**Merkle leaf:** deterministic canonical byte encoding of domain separator, program/asset, epoch, investor opaque ID, gross entitlement, withheld amount, net entitlement, currency, decimals, receipt nonce. `net = gross - withheld`; prevent ambiguous serialization and cross-epoch replay. Use SHA-256 and a clearly specified pair ordering. Publish a test vector. The SDK verifies leaf inclusion; it does **not** prove payment, rent receipt or tax remittance.

**Acceptance:** investor verifies receipt/proof against finalized epoch root; changing one amount fails proof; same proof against different epoch fails; totals arithmetic checked; correction visible and does not overwrite original.

**Tax policy:** do not calculate tax rates on-chain or assign sections such as 194-I/194-IA/194LBA without determining the actual legal instrument and payer/payee circumstances. Store tax category/evidence commitments as issuer assertions only; no universal rate.

## 5. Cross-module workflow
1. Issuer creates asset `BKC-DEMO-001` with fake issuer and asset metadata.
2. Issuer registers fake valuation and title-search document commitments; approved fictional verifier attests.
3. Registrar opens offering referencing asset and terms hash; registers eight synthetic people, invites six, accepts four applications, allots three.
4. Optional demo token mint may represent a synthetic claim only; **no legal rights implied**. Registry works without mint or MPL-3643.
5. Issuer creates a fictional rent distribution epoch using a deterministic off-chain snapshot; commits Merkle root.
6. Investor downloads a synthetic receipt and verifies proof; demo shows tampered proof failure.
7. Issuer registers a new valuation version; auditor inspects history.

## 6. SDK expectations
Package `@indian-rwa/registry-sdk` (local workspace, do not publish automatically). Functions: `createAsset`, `authorizeVerifier`, `addDocumentVersion`, `attestDocument`, `createOffering`, `registerOfferee`, `recordInvitation`, `recordApplication`, `recordAllotment`, `createDistributionEpoch`, `finalizeDistributionEpoch`, `buildDistributionTree`, `getDistributionProof`, `verifyDistributionProof`, and typed fetch helpers. Use verified Anchor client conventions and explicit signer requirements. Prefer stable return types and no magic global state.

## 7. UI expectations
Simple Next.js demo with Issuer, Registrar, Investor and Auditor tabs or pages. Show simulated/demo labels prominently. Display asset document history, offering funnel, epoch totals and receipt verification. The UI must never display a green "legally compliant", "title verified" or "tax paid" badge solely from on-chain data.

## 8. Acceptance criteria
- `anchor build` succeeds in documented environment.
- Tests exercise all three registries, role boundaries, uniqueness and negative cases.
- README enables a fresh developer to run local validator, deploy programs, seed fictional data and open demo.
- Shared asset reference is enforced on-chain across registries.
- SDK verifies Merkle proofs locally with a known test vector.
- No PII, live funds or legal-compliance claims.
- Explicit list of what remains unverified or mocked.

## 9. Delivery priority
P0 order: shared asset + document registry -> placement -> distribution -> SDK -> integration tests -> demo UI. If time runs out, prioritize executable programs and end-to-end tests over styling.
