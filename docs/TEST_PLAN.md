# Test Plan

Use Anchor tests with a local validator; LiteSVM optional if available. Every test should assert account state and expected errors, not just transaction success.

| ID | Test | Expected |
|---|---|---|
| P01 | Create asset with issuer | Correct PDA and authority |
| P02 | Duplicate asset | Reject |
| P03 | Unauthorized verifier authorization | Reject |
| P04 | Register document v1, v2 | Both immutable; v2 links to v1 |
| P05 | Skip document version / invalid predecessor | Reject |
| P06 | Unauthorized or revoked verifier attestation | Reject |
| P07 | Wrong asset substituted in document | Reject |
| O01 | Create offering linked to asset | Correct issuer/asset |
| O02 | Register one offeree twice | Reject |
| O03 | Invite, apply, allot | Correct transitions and counters |
| O04 | Allot before invitation/application | Reject |
| O05 | Registrar impersonation | Reject |
| O06 | Close then mutate offering | Reject |
| O07 | One synthetic person mapped to two wallets | Count remains one (off-chain mapping demo) |
| D01 | Create and finalize epoch | Correct amounts/root/status |
| D02 | Withheld > distributable | Reject |
| D03 | Arithmetic overflow/underflow | Reject |
| D04 | Rewrite finalized epoch | Reject |
| D05 | Correction references prior epoch | Both records visible |
| D06 | Valid receipt Merkle proof | SDK returns true |
| D07 | Tampered amount, wrong epoch/asset, wrong nonce | SDK returns false |
| X01 | End-to-end asset -> offering -> epoch | All three linked |
| X02 | Unrelated asset/mint substitution | Reject |
| X03 | No real PII in fixtures or logs | Static inspection |

## Golden Merkle test vectors
Create `sdk/test-vectors/merkle.json` containing fixed leaf inputs, canonical bytes in hex, leaf hashes, root, proof and a tampered example. Ensure test vector is generated once and validated independently where feasible.

## Run and report
- `cargo fmt --check`
- `anchor build`
- `anchor test` (or documented equivalent)
- SDK unit tests and TypeScript typecheck
- UI build (if implemented)

Document environment/version, commands, pass/fail/skip counts and blockers. Never substitute mocked SDK calls for end-to-end contract tests without explicit labeling.
