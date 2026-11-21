import * as anchor from "@coral-xyz/anchor";
import { strict as assert } from "node:assert";
import { randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createRegistryClient, hashDocument } from "../sdk/src";

describe("placement and distribution registries", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const idl = JSON.parse(readFileSync(resolve(__dirname, "../target/idl/indian_rwa_registry.json"), "utf8")) as anchor.Idl;
  const client = createRegistryClient({ idl, provider });
  const issuer = provider.wallet.publicKey;
  const commitment = () => hashDocument(randomBytes(32));
  const id = () => randomBytes(32);

  it("enforces registrar, unique offerees, event order, totals, finalization, and correction chain", async () => {
    const assetId = id();
    const asset = client.assetPda(issuer, assetId);
    await client.createAsset({ issuer, assetId, metadataCommitment: commitment() });

    const registrar = Keypair.generate();
    const airdrop = await provider.connection.requestAirdrop(registrar.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(airdrop, "confirmed");
    await client.authorizeRegistrar({ issuer, asset, registrar: registrar.publicKey });
    const offeringId = id();
    const offering = client.offeringPda(asset, offeringId);
    await client.createOffering({ issuer, asset, offeringId, fiscalPeriod: new Uint8Array(16), termsCommitment: commitment(), administrativeOffereeLimit: 2 });

    const firstRef = id();
    const firstOfferee = client.offereePda(offering, firstRef);
    await client.registerOfferee({ registrar: registrar.publicKey, asset, offering, offereeRef: firstRef, signers: [registrar] });
    await assert.rejects(() => client.registerOfferee({ registrar: registrar.publicKey, asset, offering, offereeRef: firstRef, signers: [registrar] }));
    await assert.rejects(() => client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee,
      event: "allot", commitment: commitment(), allotmentAmount: 50n, signers: [registrar] }));
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee, event: "invite", commitment: commitment(), signers: [registrar] });
    await assert.rejects(() => client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee,
      event: "invite", commitment: commitment(), signers: [registrar] }));
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee, event: "apply", commitment: commitment(), signers: [registrar] });
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee,
      event: "allot", commitment: commitment(), allotmentAmount: 50n, signers: [registrar] });
    const secondRef = id();
    await client.registerOfferee({ registrar: registrar.publicKey, asset, offering, offereeRef: secondRef, signers: [registrar] });
    await assert.rejects(() => client.registerOfferee({ registrar: registrar.publicKey, asset, offering, offereeRef: id(), signers: [registrar] }));
    const funnel = await client.fetchOffering(offering);
    assert.equal(funnel.registeredCount, 2);
    assert.equal(funnel.invitedCount, 1);
    assert.equal(funnel.appliedCount, 1);
    assert.equal(funnel.allottedCount, 1);

    await client.closeOffering({ issuer, offering });
    await assert.rejects(() => client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree: firstOfferee,
      event: "withdraw", commitment: commitment(), signers: [registrar] }));

    const epochId = 77n;
    const epoch = client.epochPda(asset, epochId);
    const baseEpoch = {
      issuer, asset, epochId, currencyCode: "INR", decimals: 2, recordDate: BigInt(Math.floor(Date.now() / 1000)),
      snapshotCommitment: commitment(), gross: 1000n, expensesAndReserves: 100n, distributable: 900n,
      totalWithheld: 100n, totalNet: 800n, merkleRoot: commitment(), evidenceManifestCommitment: commitment(),
    };
    await assert.rejects(() => client.createDistributionEpoch({ ...baseEpoch, distributable: 899n }));
    await client.createDistributionEpoch(baseEpoch);
    await assert.rejects(() => client.recordCorrection({ issuer, epoch, correctionIndex: 0, currencyCode: "INR", decimals: 2,
      gross: 1000n, expensesAndReserves: 100n, distributable: 900n, totalWithheld: 901n, totalNet: 0n,
      merkleRoot: commitment(), evidenceManifestCommitment: commitment(), reasonCommitment: commitment() }));
    await client.finalizeDistributionEpoch({ issuer, epoch });
    await assert.rejects(() => client.finalizeDistributionEpoch({ issuer, epoch }));
    const correction0 = client.correctionPda(epoch, 0);
    await client.recordCorrection({ issuer, epoch, correctionIndex: 0, currencyCode: "INR", decimals: 2,
      gross: 1000n, expensesAndReserves: 100n, distributable: 900n, totalWithheld: 100n, totalNet: 800n,
      merkleRoot: commitment(), evidenceManifestCommitment: commitment(), reasonCommitment: commitment() });
    await assert.rejects(() => client.recordCorrection({ issuer, epoch, correctionIndex: 2, currencyCode: "INR", decimals: 2,
      gross: 1000n, expensesAndReserves: 100n, distributable: 900n, totalWithheld: 100n, totalNet: 800n,
      merkleRoot: commitment(), evidenceManifestCommitment: commitment(), reasonCommitment: commitment(), previousCorrection: correction0 }));
  });

  it("does not create an offering against an unrelated or absent asset account", async () => {
    const unrelated = Keypair.generate().publicKey;
    await assert.rejects(() => client.createOffering({ issuer, asset: unrelated, offeringId: id(),
      fiscalPeriod: new Uint8Array(16), termsCommitment: commitment() }));
  });
});
