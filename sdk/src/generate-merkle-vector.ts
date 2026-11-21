import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { buildDistributionTree, canonicalReceiptBytes, getDistributionProof, type DistributionReceipt } from "./distribution";

const receipts: DistributionReceipt[] = [0n, 1n, 2n].map((nonce) => ({
  programId: new Uint8Array(32).fill(1),
  asset: new Uint8Array(32).fill(2),
  epochId: 17n,
  investorRef: new Uint8Array(32).fill(Number(nonce) + 10),
  gross: 12500n,
  withheld: 1250n,
  net: 11250n,
  currencyCode: "INR",
  decimals: 2,
  nonce,
}));
const tree = buildDistributionTree(receipts);
const vector = {
  specification: "indian-rwa-distribution-receipt-v1",
  leafEncoding: "UTF-8 domain prefix with NUL, fixed-width raw keys, unsigned integers little-endian, ASCII currency, decimals u8",
  nodeEncoding: "SHA-256(0x01 || lexicographically-smaller-hash || lexicographically-larger-hash); duplicate final node on odd levels",
  receipt: {
    programIdHex: Buffer.from(receipts[1].programId).toString("hex"),
    assetHex: Buffer.from(receipts[1].asset).toString("hex"),
    epochId: "17",
    investorRefHex: Buffer.from(receipts[1].investorRef).toString("hex"),
    gross: "12500", withheld: "1250", net: "11250", currencyCode: "INR", decimals: 2, nonce: "1",
  },
  canonicalBytesHex: Buffer.from(canonicalReceiptBytes(receipts[1])).toString("hex"),
  leafHashHex: Buffer.from(tree.leaves[1]).toString("hex"),
  rootHex: Buffer.from(tree.root).toString("hex"),
  proofHex: getDistributionProof(tree, 1).map((node) => Buffer.from(node).toString("hex")),
  tamperedNet: "11249",
};
writeFileSync(resolve(__dirname, "../test-vectors/merkle.json"), `${JSON.stringify(vector, null, 2)}\n`);
