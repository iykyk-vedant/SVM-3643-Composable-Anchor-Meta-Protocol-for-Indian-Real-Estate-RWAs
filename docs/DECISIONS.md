# Design decisions

## One Anchor program, three registry modules

The property, placement, and distribution registries are separate Rust modules in one deployable Anchor program. This keeps the shared asset relationship enforceable without introducing cross-program account decoding and CPI overhead. The account types and instruction handlers remain grouped by registry so the modules can be split into separate programs later if independent upgrades become necessary.

## Append-only evidence

Document versions, offeree events, distribution epochs, and corrections use separate records. Existing evidence is not silently overwritten. Corrections link to the record they supersede.

## Off-chain judgments

Identity mapping, document authenticity, title, eligibility, legal interpretation, tax treatment, accounting, and payment settlement remain off-chain. On-chain commitments support integrity and history checks; they do not establish those facts.
