import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import type { IndianRwaRegistry } from "../target/types/indian_rwa_registry";

describe("property registry", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.IndianRwaRegistry as anchor.Program<IndianRwaRegistry>;
  const issuer = provider.wallet.publicKey;
  const assetId = Uint8Array.from({ length: 32 }, (_, i) => i + 1);
  const metadata = new Uint8Array(32).fill(7);
  const documentKey = new Uint8Array(32).fill(9);

  const assetPda = (authority: PublicKey, id: Uint8Array) => PublicKey.findProgramAddressSync(
    [Buffer.from("asset"), authority.toBuffer(), Buffer.from(id)], program.programId,
  )[0];

  it("creates an issuer-bound asset; rejects a duplicate PDA", async () => {
    const asset = assetPda(issuer, assetId);
    await program.methods.createAsset([...assetId], [...metadata], null)
      .accounts({ issuer, asset }).rpc();
    const state = await program.account.assetRegistry.fetch(asset);
    if (!state.issuer.equals(issuer)) throw new Error("issuer mismatch");
    if (state.assetId.some((byte, i) => byte !== assetId[i])) throw new Error("asset id mismatch");

    await expectFailure(() => program.methods.createAsset([...assetId], [...metadata], null)
      .accounts({ issuer, asset }).rpc());
  });

  it("stores v1 and v2 as separate records; rejects a skipped version", async () => {
    const asset = assetPda(issuer, assetId);
    const document = (version: bigint) => PublicKey.findProgramAddressSync(
      [Buffer.from("doc"), asset.toBuffer(), Buffer.from(documentKey), Buffer.from(new anchor.BN(version.toString()).toArray("le", 8))],
      program.programId,
    )[0];
    const v1 = document(1n);
    await program.methods.addDocumentVersion([...documentKey], new anchor.BN(1), { valuation: {} },
      [...new Uint8Array(32).fill(1)], [...new Uint8Array(32).fill(2)])
      .accounts({ issuer, asset, document: v1, previousDocument: null }).rpc();

    const impostor = Keypair.generate();
    const airdrop = await provider.connection.requestAirdrop(impostor.publicKey, anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(airdrop, "confirmed");
    const unauthorizedDocument = PublicKey.findProgramAddressSync(
      [Buffer.from("doc"), asset.toBuffer(), Buffer.from(new Uint8Array(32).fill(8)), Buffer.from(new anchor.BN(1).toArray("le", 8))],
      program.programId,
    )[0];
    await expectFailure(() => program.methods.addDocumentVersion([...new Uint8Array(32).fill(8)], new anchor.BN(1), { valuation: {} },
      [...new Uint8Array(32).fill(1)], [...new Uint8Array(32).fill(2)])
      .accounts({ issuer: impostor.publicKey, asset, document: unauthorizedDocument, previousDocument: null })
      .signers([impostor]).rpc());

    const v2 = document(2n);
    await program.methods.addDocumentVersion([...documentKey], new anchor.BN(2), { valuation: {} },
      [...new Uint8Array(32).fill(3)], [...new Uint8Array(32).fill(4)])
      .accounts({ issuer, asset, document: v2, previousDocument: v1 }).rpc();
    const [first, second] = await Promise.all([
      program.account.documentVersion.fetch(v1), program.account.documentVersion.fetch(v2),
    ]);
    if (!second.previousVersion?.equals(v1) || first.documentHash[0] === second.documentHash[0]) {
      throw new Error("document version chain not preserved");
    }

    const v4 = document(4n);
    await expectFailure(() => program.methods.addDocumentVersion([...documentKey], new anchor.BN(4), { valuation: {} },
      [...new Uint8Array(32).fill(5)], [...new Uint8Array(32).fill(6)])
      .accounts({ issuer, asset, document: v4, previousDocument: v2 }).rpc());

    const verifier = Keypair.generate();
    const verifierAirdrop = await provider.connection.requestAirdrop(verifier.publicKey, anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(verifierAirdrop, "confirmed");
    const verifierAuthorization = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier"), asset.toBuffer(), verifier.publicKey.toBuffer()], program.programId,
    )[0];
    await program.methods.authorizeVerifier(2).accounts({ issuer, verifier: verifier.publicKey, asset,
      authorization: verifierAuthorization, systemProgram: anchor.web3.SystemProgram.programId }).rpc();
    const attestation = PublicKey.findProgramAddressSync(
      [Buffer.from("attestation"), v2.toBuffer(), verifier.publicKey.toBuffer()], program.programId,
    )[0];
    await program.methods.attestDocument([...new Uint8Array(32).fill(13)], { reviewed: {} })
      .accounts({ verifier: verifier.publicKey, authorization: verifierAuthorization, asset, document: v2,
        attestation, systemProgram: anchor.web3.SystemProgram.programId }).signers([verifier]).rpc();
    await expectFailure(() => program.methods.attestDocument([...new Uint8Array(32).fill(14)], { reviewed: {} })
      .accounts({ verifier: verifier.publicKey, authorization: verifierAuthorization, asset, document: v2,
        attestation, systemProgram: anchor.web3.SystemProgram.programId }).signers([verifier]).rpc());

    const v3 = document(3n);
    await program.methods.addDocumentVersion([...documentKey], new anchor.BN(3), { valuation: {} },
      [...new Uint8Array(32).fill(7)], [...new Uint8Array(32).fill(8)])
      .accounts({ issuer, asset, document: v3, previousDocument: v2 }).rpc();
    await program.methods.revokeVerifier().accounts({ issuer, verifier: verifier.publicKey, asset,
      authorization: verifierAuthorization, systemProgram: anchor.web3.SystemProgram.programId }).rpc();
    const revokedAttestation = PublicKey.findProgramAddressSync(
      [Buffer.from("attestation"), v3.toBuffer(), verifier.publicKey.toBuffer()], program.programId,
    )[0];
    await expectFailure(() => program.methods.attestDocument([...new Uint8Array(32).fill(15)], { reviewed: {} })
      .accounts({ verifier: verifier.publicKey, authorization: verifierAuthorization, asset, document: v3,
        attestation: revokedAttestation, systemProgram: anchor.web3.SystemProgram.programId }).signers([verifier]).rpc());
  });
});

async function expectFailure(action: () => Promise<unknown>): Promise<void> {
  try { await action(); } catch { return; }
  throw new Error("expected transaction to fail");
}
