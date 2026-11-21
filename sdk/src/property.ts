import { sha256 } from "@noble/hashes/sha256";

const DOCUMENT_DOMAIN = new TextEncoder().encode("indian-rwa:document:v1\0");

/** Hash local document bytes with protocol domain separation. This proves byte integrity only. */
export function hashDocument(bytes: Uint8Array): Uint8Array {
  const input = new Uint8Array(DOCUMENT_DOMAIN.length + bytes.length);
  input.set(DOCUMENT_DOMAIN);
  input.set(bytes, DOCUMENT_DOMAIN.length);
  return sha256(input);
}
