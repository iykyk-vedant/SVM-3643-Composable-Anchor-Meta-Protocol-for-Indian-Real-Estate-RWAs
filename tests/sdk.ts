import { strict as assert } from "node:assert";
import { hashDocument } from "../sdk/src/property";

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
