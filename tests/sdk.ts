import { strict as assert } from "node:assert";
import vector from "../sdk/test-vectors/merkle.json";
import { hashDocument } from "../sdk/src/property";
import {
  buildDistributionTree, canonicalReceiptBytes, getDistributionProof, verifyDistributionProof,
  type DistributionReceipt,
} from "../sdk/src/distribution";

describe("document hashing", () => {
  it("uses a domain-separated SHA-256 commitment", () => {
    const hash = hashDocument(new TextEncoder().encode("fictional valuation v1"));
    assert.equal(hash.length, 32);
    assert.notDeepEqual(hash, hashDocument(new TextEncoder().encode("fictional valuation v2")));
  });

  it("distinguishes empty data from an empty commitment", () => {
    assert.notDeepEqual(hashDocument(new Uint8Array()), new Uint8Array(32));
  });
});

describe("distribution receipt Merkle proofs", () => {
  const receipt = (nonce: bigint): DistributionReceipt => ({
    programId: new Uint8Array(32).fill(1), asset: new Uint8Array(32).fill(2), epochId: 17n,
    investorRef: new Uint8Array(32).fill(Number(nonce) + 10), gross: 12500n, withheld: 1250n,
    net: 11250n, currencyCode: "INR", decimals: 2, nonce,
  });

  it("verifies inclusion and rejects amount or epoch tampering", () => {
    const receipts = [receipt(0n), receipt(1n), receipt(2n)];
    const tree = buildDistributionTree(receipts);
    const proof = getDistributionProof(tree, 1);
    assert.equal(verifyDistributionProof(receipts[1], proof, tree.root), true);
    assert.equal(verifyDistributionProof({ ...receipts[1], net: 11249n }, proof, tree.root), false);
    assert.equal(verifyDistributionProof({ ...receipts[1], epochId: 18n }, proof, tree.root), false);
  });

  it("rejects ambiguous or inconsistent canonical receipt inputs", () => {
    assert.equal(canonicalReceiptBytes(receipt(3n)).length, 175);
    assert.throws(() => canonicalReceiptBytes({ ...receipt(3n), currencyCode: "inr" }));
    assert.throws(() => canonicalReceiptBytes({ ...receipt(3n), net: 1n }));
  });

  it("matches the published canonical byte and proof vector", () => {
    const input: DistributionReceipt = {
      programId: Uint8Array.from(Buffer.from(vector.receipt.programIdHex, "hex")),
      asset: Uint8Array.from(Buffer.from(vector.receipt.assetHex, "hex")),
      epochId: BigInt(vector.receipt.epochId),
      investorRef: Uint8Array.from(Buffer.from(vector.receipt.investorRefHex, "hex")),
      gross: BigInt(vector.receipt.gross), withheld: BigInt(vector.receipt.withheld),
      net: BigInt(vector.receipt.net), currencyCode: vector.receipt.currencyCode,
      decimals: vector.receipt.decimals, nonce: BigInt(vector.receipt.nonce),
    };
    const tree = buildDistributionTree([receipt(0n), input, receipt(2n)]);
    assert.equal(Buffer.from(canonicalReceiptBytes(input)).toString("hex"), vector.canonicalBytesHex);
    assert.equal(Buffer.from(tree.leaves[1]).toString("hex"), vector.leafHashHex);
    assert.equal(Buffer.from(tree.root).toString("hex"), vector.rootHex);
    assert.deepEqual(getDistributionProof(tree, 1).map((node) => Buffer.from(node).toString("hex")), vector.proofHex);
    assert.equal(verifyDistributionProof(input, getDistributionProof(tree, 1), tree.root), true);
  });
});
