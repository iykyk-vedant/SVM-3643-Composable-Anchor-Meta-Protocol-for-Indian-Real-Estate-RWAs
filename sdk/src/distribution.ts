import { sha256 } from "@noble/hashes/sha256";

const LEAF_DOMAIN = new TextEncoder().encode("indian-rwa:distribution-receipt:v1\0");
const NODE_PREFIX = Uint8Array.of(1);

export type DistributionReceipt = {
  programId: Uint8Array;
  asset: Uint8Array;
  epochId: bigint;
  investorRef: Uint8Array;
  gross: bigint;
  withheld: bigint;
  net: bigint;
  currencyCode: string;
  decimals: number;
  nonce: bigint;
};

export type DistributionTree = { leaves: Uint8Array[]; root: Uint8Array; levels: Uint8Array[][] };

export function canonicalReceiptBytes(receipt: DistributionReceipt): Uint8Array {
  assertBytes(receipt.programId, 32, "programId");
  assertBytes(receipt.asset, 32, "asset");
  assertBytes(receipt.investorRef, 32, "investorRef");
  assertU64(receipt.epochId, "epochId");
  assertU64(receipt.gross, "gross");
  assertU64(receipt.withheld, "withheld");
  assertU64(receipt.net, "net");
  assertU64(receipt.nonce, "nonce");
  if (receipt.net !== receipt.gross - receipt.withheld || receipt.withheld > receipt.gross) {
    throw new Error("receipt net must equal gross minus withheld");
  }
  if (!/^[A-Z]{3}$/.test(receipt.currencyCode)) throw new Error("currencyCode must be three uppercase letters");
  if (!Number.isInteger(receipt.decimals) || receipt.decimals < 0 || receipt.decimals > 255) {
    throw new Error("decimals must fit in u8");
  }
  const code = new TextEncoder().encode(receipt.currencyCode);
  return concat(
    LEAF_DOMAIN, receipt.programId, receipt.asset, u64le(receipt.epochId), receipt.investorRef,
    u64le(receipt.gross), u64le(receipt.withheld), u64le(receipt.net), code,
    Uint8Array.of(receipt.decimals), u64le(receipt.nonce),
  );
}

export function hashReceiptLeaf(receipt: DistributionReceipt): Uint8Array {
  const bytes = canonicalReceiptBytes(receipt);
  return sha256(bytes);
}

export function buildDistributionTree(receipts: DistributionReceipt[]): DistributionTree {
  if (receipts.length === 0) throw new Error("at least one receipt is required");
  const leaves = receipts.map(hashReceiptLeaf);
  let level = leaves;
  const levels = [level];
  while (level.length > 1) {
    const next: Uint8Array[] = [];
    for (let i = 0; i < level.length; i += 2) {
      const left = level[i];
      const right = level[i + 1] ?? left;
      next.push(hashNode(left, right));
    }
    level = next;
    levels.push(level);
  }
  return { leaves, root: level[0], levels };
}

export function getDistributionProof(tree: DistributionTree, leafIndex: number): Uint8Array[] {
  if (!Number.isInteger(leafIndex) || leafIndex < 0 || leafIndex >= tree.leaves.length) {
    throw new Error("leafIndex is out of range");
  }
  const proof: Uint8Array[] = [];
  let index = leafIndex;
  for (let depth = 0; depth < tree.levels.length - 1; depth += 1) {
    const level = tree.levels[depth];
    const siblingIndex = index ^ 1;
    proof.push(level[siblingIndex] ?? level[index]);
    index = Math.floor(index / 2);
  }
  return proof;
}

export function verifyDistributionProof(
  receipt: DistributionReceipt,
  proof: Uint8Array[],
  expectedRoot: Uint8Array,
): boolean {
  if (expectedRoot.length !== 32 || proof.some((node) => node.length !== 32)) return false;
  try {
    let hash = hashReceiptLeaf(receipt);
    for (const sibling of proof) hash = hashNode(hash, sibling);
    return equalBytes(hash, expectedRoot);
  } catch {
    return false;
  }
}

function hashNode(left: Uint8Array, right: Uint8Array): Uint8Array {
  const [first, second] = compareBytes(left, right) <= 0 ? [left, right] : [right, left];
  return sha256(concat(NODE_PREFIX, first, second));
}

function u64le(value: bigint): Uint8Array {
  const bytes = new Uint8Array(8);
  let remaining = value;
  for (let i = 0; i < 8; i += 1) {
    bytes[i] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  return bytes;
}

function assertU64(value: bigint, label: string): void {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) throw new Error(`${label} must fit in u64`);
}

function assertBytes(value: Uint8Array, length: number, label: string): void {
  if (!(value instanceof Uint8Array) || value.length !== length) throw new Error(`${label} must be ${length} bytes`);
}

function concat(...parts: Uint8Array[]): Uint8Array {
  const result = new Uint8Array(parts.reduce((size, part) => size + part.length, 0));
  let offset = 0;
  for (const part of parts) { result.set(part, offset); offset += part.length; }
  return result;
}

function compareBytes(a: Uint8Array, b: Uint8Array): number {
  for (let i = 0; i < a.length; i += 1) if (a[i] !== b[i]) return a[i] - b[i];
  return 0;
}

function equalBytes(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((byte, i) => byte === b[i]);
}
