import { AnchorProvider, type Idl } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  buildDistributionTree, createRegistryClient, getDistributionProof, hashDocument, verifyDistributionProof,
  type DistributionReceipt,
} from "./index";

async function main(): Promise<void> {
  const provider = AnchorProvider.env();
  const idl = JSON.parse(readFileSync(resolve(__dirname, "../../target/idl/indian_rwa_registry.json"), "utf8")) as Idl;
  const client = createRegistryClient({ idl, provider });
  const issuer = provider.wallet.publicKey;
  const assetId = randomBytes(32);
  const metadata = hashDocument(Buffer.from("Fictional BKC commercial property metadata; demo only."));
  const asset = client.assetPda(issuer, assetId);
  const verifier = Keypair.generate();
  const registrar = Keypair.generate();
  await Promise.all([verifier, registrar].map(async (keypair) => {
    const signature = await provider.connection.requestAirdrop(keypair.publicKey, 1_000_000_000);
    await provider.connection.confirmTransaction(signature, "confirmed");
  }));

  await client.createAsset({ issuer, assetId, metadataCommitment: metadata });
  const documentKey = randomBytes(32);
  const valuationV1 = hashDocument(Buffer.from("Fictional valuation report v1."));
  const valuationV2 = hashDocument(Buffer.from("Fictional valuation report v2, corrected area assumption."));
  const manifest = hashDocument(Buffer.from("Fictional document manifest."));
  const v1 = client.documentPda(asset, documentKey, 1n);
  const v2 = client.documentPda(asset, documentKey, 2n);
  await client.addDocumentVersion({ issuer, asset, documentKey, version: 1n, documentType: "valuation", documentHash: valuationV1, manifestHash: manifest });
  await client.addDocumentVersion({ issuer, asset, documentKey, version: 2n, documentType: "valuation", documentHash: valuationV2, manifestHash: manifest, previousDocument: v1 });
  await client.authorizeVerifier({ issuer, asset, verifier: verifier.publicKey, scope: 1 << 1 });
  await client.attestDocument({ verifier: verifier.publicKey, asset, document: v2,
    reviewCommitment: hashDocument(Buffer.from("Fictional review note.")), outcome: "reviewed", signers: [verifier] });

  const offeringId = randomBytes(32);
  const fiscalPeriod = new Uint8Array(16);
  fiscalPeriod.set(Buffer.from("FY26-27"));
  const offering = client.offeringPda(asset, offeringId);
  await client.authorizeRegistrar({ issuer, asset, registrar: registrar.publicKey });
  await client.createOffering({ issuer, asset, offeringId, fiscalPeriod,
    termsCommitment: hashDocument(Buffer.from("Fictional offering terms.")), administrativeOffereeLimit: 12 });

  const references = Array.from({ length: 8 }, () => randomBytes(32));
  for (const offereeRef of references) {
    await client.registerOfferee({ registrar: registrar.publicKey, asset, offering, offereeRef, signers: [registrar] });
  }
  const eventCommitment = () => hashDocument(randomBytes(32));
  for (let i = 0; i < 6; i += 1) {
    const offeree = client.offereePda(offering, references[i]);
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree, event: "invite", commitment: eventCommitment(), signers: [registrar] });
  }
  for (let i = 0; i < 4; i += 1) {
    const offeree = client.offereePda(offering, references[i]);
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree, event: "apply", commitment: eventCommitment(), signers: [registrar] });
  }
  for (let i = 0; i < 3; i += 1) {
    const offeree = client.offereePda(offering, references[i]);
    await client.recordOffereeEvent({ registrar: registrar.publicKey, asset, offering, offeree, event: "allot", commitment: eventCommitment(), allotmentAmount: 500n, signers: [registrar] });
  }

  const epochId = 202611n;
  const receiptInputs: DistributionReceipt[] = [
    [100_000n, 10_000n, 90_000n], [80_000n, 8_000n, 72_000n], [70_000n, 7_000n, 63_000n],
  ].map(([gross, withheld, net], index) => ({
    programId: client.program.programId.toBytes(), asset: asset.toBytes(), epochId,
    investorRef: references[index], gross, withheld, net, currencyCode: "INR", decimals: 2, nonce: BigInt(index + 1),
  }));
  const tree = buildDistributionTree(receiptInputs);
  const epoch = client.epochPda(asset, epochId);
  const snapshot = hashDocument(Buffer.from("Fictional record-date snapshot."));
  const evidence = hashDocument(Buffer.from("Fictional distribution evidence manifest."));
  await client.createDistributionEpoch({ issuer, asset, epochId, currencyCode: "INR", decimals: 2,
    recordDate: BigInt(Math.floor(Date.now() / 1000)), snapshotCommitment: snapshot,
    gross: 300_000n, expensesAndReserves: 50_000n, distributable: 250_000n,
    totalWithheld: 25_000n, totalNet: 225_000n, merkleRoot: tree.root, evidenceManifestCommitment: evidence });
  await client.finalizeDistributionEpoch({ issuer, epoch });

  const proof = getDistributionProof(tree, 1);
  const valid = verifyDistributionProof(receiptInputs[1], proof, tree.root);
  const tampered = verifyDistributionProof({ ...receiptInputs[1], net: 71_999n }, proof, tree.root);
  const offeringState = await client.fetchOffering(offering);
  console.log(JSON.stringify({
    demoOnly: true,
    asset: asset.toBase58(),
    documentVersions: [v1.toBase58(), v2.toBase58()],
    offering: { address: offering.toBase58(), registered: offeringState.registeredCount, invited: offeringState.invitedCount,
      applied: offeringState.appliedCount, allotted: offeringState.allottedCount },
    distribution: { epoch: epoch.toBase58(), merkleRoot: Buffer.from(tree.root).toString("hex"),
      validProof: valid, tamperedProof: tampered },
    caveat: "Synthetic record commitments and inclusion proofs are not title, compliance, bank settlement, or tax remittance evidence.",
  }, null, 2));
  if (!valid || tampered) throw new Error("receipt proof checks did not match expected demo results");
}

void main().catch((error: unknown) => { console.error(error); process.exitCode = 1; });
