# Codex Agent Instructions — Indian RWA Registry Protocol

## Mission
Implement the exact MVP described in `PRD.md`. This is a five-day hackathon prototype: one open-source Solana protocol containing three interoperable registries, a TypeScript SDK, and a fictional Indian commercial-property demonstration. Do not build the older SVM-3643 six-module protocol.

## Read first, in order
1. `PRD.md`
2. `docs/ARCHITECTURE.md`
3. `docs/SECURITY.md`
4. `docs/TEST_PLAN.md`
5. `docs/IMPLEMENTATION_PLAN.md`
6. `docs/EXTERNAL_DEPENDENCIES.md`

## Non-negotiable rules
- Build actual compiling Rust/Anchor code, not pseudocode or mocked smart-contract success paths.
- Verify versions and APIs against installed tools and official source code before use. Do not invent Metaplex MPL-3643, SAS, Token ACL, Anchor, or Solana SDK APIs. External token-compliance integration is OPTIONAL and not a blocker for the registry MVP.
- Use fictional asset, issuer, verifier, investor and payment data. No PAN, Aadhaar, actual KYC, real property title claims or live banking.
- Keep all legal and tax determinations OFF-CHAIN and issuer-/professional-attested. Never claim legal compliance is guaranteed by the protocol.
- Never store raw identity documents, tax IDs, private investor data or confidential contracts on-chain. Avoid unsalted low-entropy identifier hashes.
- Require explicit authorization for every mutation. Validate account owners, PDA seeds, relationships, state transitions, arithmetic, uniqueness and replay protection.
- A document hash verifies byte integrity, not authenticity or legal title. An issuer-signed receipt is not proof of bank settlement or tax remittance.
- Offeree counts are not token-holder counts. Do not hardcode a universal 200-person rule, TDS rate, tax treatment or eligibility decision.
- Design for corrections using append-only records and supersession; never silently rewrite evidence.
- Avoid unnecessary features: no DEX, public token marketplace, on-chain KYC, real tax computation, real money, real title transfer, yield token economics, governance multisig or complex token ACL integration in P0.
- Run `cargo fmt`, `cargo test` where applicable, `anchor build`, and integration tests; report commands and results truthfully. Never claim tests passed if not executed.
- Make pragmatic decisions when details are unspecified; record them in `docs/DECISIONS.md`. Ask only if blocked by an irreversible architecture choice.

## Required working rhythm
1. Inspect repo, toolchain, official docs and write a short implementation checklist.
2. Implement shared asset identity and property registry first.
3. Implement placement registry and distribution registry, with tests after each.
4. Implement TypeScript SDK and end-to-end fictional demo.
5. Run negative tests and document all limitations.

## Definition of done
Three functional programs (or three logically separated modules in a single program if justified and documented), a tested TS SDK, a runnable fictional property lifecycle demo, README with exact commands, and a security/limitations section. Prioritize correctness and demo completeness over UI polish.
