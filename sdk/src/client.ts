import { BN, type Idl, Program, type Provider } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, type Signer } from "@solana/web3.js";

export type ClientConfig = { idl: Idl; provider: Provider };
export type DocumentTypeName = "titleSearch" | "valuation" | "spvEvidence" | "reraRecord" | "leaseStatement" | "other";
export type OffereeEventName = "invite" | "apply" | "allot" | "reject" | "withdraw";
export type RegistryAddresses = {
  asset: PublicKey;
  document?: PublicKey;
  verifierAuthorization?: PublicKey;
  attestation?: PublicKey;
  registrarAuthorization?: PublicKey;
  offering?: PublicKey;
  offeree?: PublicKey;
  offereeEvent?: PublicKey;
  epoch?: PublicKey;
  correction?: PublicKey;
};

type DynamicProgram = {
  programId: PublicKey;
  methods: Record<string, (...args: any[]) => any>;
  account: Record<string, any>;
};

export function createRegistryClient(config: ClientConfig): RegistryClient {
  return new RegistryClient(config.idl, config.provider);
}

export function deriveRegistryAddresses(programId: PublicKey, assetId: Uint8Array, issuer: PublicKey): RegistryAddresses {
  assert32(assetId, "assetId");
  const [asset] = PublicKey.findProgramAddressSync([Buffer.from("asset"), issuer.toBuffer(), Buffer.from(assetId)], programId);
  return { asset };
}

export class RegistryClient {
  readonly program: DynamicProgram;

  constructor(idl: Idl, provider: Provider) {
    this.program = new Program(idl as any, provider) as unknown as DynamicProgram;
  }

  assetPda(issuer: PublicKey, assetId: Uint8Array): PublicKey {
    assert32(assetId, "assetId");
    return pda(this.program.programId, "asset", issuer.toBuffer(), assetId);
  }

  documentPda(asset: PublicKey, documentKey: Uint8Array, version: bigint): PublicKey {
    assert32(documentKey, "documentKey");
    return pda(this.program.programId, "doc", asset.toBuffer(), documentKey, u64(version));
  }

  verifierAuthorizationPda(asset: PublicKey, verifier: PublicKey): PublicKey {
    return pda(this.program.programId, "verifier", asset.toBuffer(), verifier.toBuffer());
  }

  attestationPda(document: PublicKey, verifier: PublicKey): PublicKey {
    return pda(this.program.programId, "attestation", document.toBuffer(), verifier.toBuffer());
  }

  registrarAuthorizationPda(asset: PublicKey, registrar: PublicKey): PublicKey {
    return pda(this.program.programId, "registrar", asset.toBuffer(), registrar.toBuffer());
  }

  offeringPda(asset: PublicKey, offeringId: Uint8Array): PublicKey {
    assert32(offeringId, "offeringId");
    return pda(this.program.programId, "offering", asset.toBuffer(), offeringId);
  }

  offereePda(offering: PublicKey, offereeRef: Uint8Array): PublicKey {
    assert32(offereeRef, "offereeRef");
    return pda(this.program.programId, "offeree", offering.toBuffer(), offereeRef);
  }

  offereeEventPda(offeree: PublicKey, index: number): PublicKey {
    return pda(this.program.programId, "offeree-event", offeree.toBuffer(), u32(index));
  }

  epochPda(asset: PublicKey, epochId: bigint): PublicKey {
    return pda(this.program.programId, "epoch", asset.toBuffer(), u64(epochId));
  }

  correctionPda(epoch: PublicKey, index: number): PublicKey {
    return pda(this.program.programId, "correction", epoch.toBuffer(), u32(index));
  }

  async createAsset(input: { issuer: PublicKey; assetId: Uint8Array; metadataCommitment: Uint8Array; mint?: PublicKey | null; signers?: Signer[] }): Promise<string> {
    assert32(input.assetId, "assetId"); assert32(input.metadataCommitment, "metadataCommitment");
    return this.program.methods.createAsset([...input.assetId], [...input.metadataCommitment], input.mint ?? null)
      .accounts({ issuer: input.issuer, asset: this.assetPda(input.issuer, input.assetId), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async addDocumentVersion(input: {
    issuer: PublicKey; asset: PublicKey; documentKey: Uint8Array; version: bigint; documentType: DocumentTypeName;
    documentHash: Uint8Array; manifestHash: Uint8Array; previousDocument?: PublicKey | null; signers?: Signer[];
  }): Promise<string> {
    assert32(input.documentKey, "documentKey"); assert32(input.documentHash, "documentHash"); assert32(input.manifestHash, "manifestHash");
    return this.program.methods.addDocumentVersion([...input.documentKey], new BN(input.version.toString()), { [input.documentType]: {} },
      [...input.documentHash], [...input.manifestHash])
      .accounts({ issuer: input.issuer, asset: input.asset, document: this.documentPda(input.asset, input.documentKey, input.version),
        previousDocument: input.previousDocument ?? null, systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async authorizeVerifier(input: { issuer: PublicKey; asset: PublicKey; verifier: PublicKey; scope: number; signers?: Signer[] }): Promise<string> {
    return this.program.methods.authorizeVerifier(input.scope)
      .accounts({ issuer: input.issuer, verifier: input.verifier, asset: input.asset,
        authorization: this.verifierAuthorizationPda(input.asset, input.verifier), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async revokeVerifier(input: { issuer: PublicKey; asset: PublicKey; verifier: PublicKey; signers?: Signer[] }): Promise<string> {
    return this.program.methods.revokeVerifier()
      .accounts({ issuer: input.issuer, verifier: input.verifier, asset: input.asset,
        authorization: this.verifierAuthorizationPda(input.asset, input.verifier), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async attestDocument(input: { verifier: PublicKey; asset: PublicKey; document: PublicKey; reviewCommitment: Uint8Array; outcome: "reviewed" | "needsCorrection"; signers?: Signer[] }): Promise<string> {
    assert32(input.reviewCommitment, "reviewCommitment");
    return this.program.methods.attestDocument([...input.reviewCommitment], { [input.outcome]: {} })
      .accounts({ verifier: input.verifier, asset: input.asset, document: input.document,
        authorization: this.verifierAuthorizationPda(input.asset, input.verifier),
        attestation: this.attestationPda(input.document, input.verifier), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async authorizeRegistrar(input: { issuer: PublicKey; asset: PublicKey; registrar: PublicKey; signers?: Signer[] }): Promise<string> {
    return this.program.methods.authorizeRegistrar()
      .accounts({ issuer: input.issuer, registrar: input.registrar, asset: input.asset,
        authorization: this.registrarAuthorizationPda(input.asset, input.registrar), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async revokeRegistrar(input: { issuer: PublicKey; asset: PublicKey; registrar: PublicKey; signers?: Signer[] }): Promise<string> {
    return this.program.methods.revokeRegistrar()
      .accounts({ issuer: input.issuer, registrar: input.registrar, asset: input.asset,
        authorization: this.registrarAuthorizationPda(input.asset, input.registrar), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async createOffering(input: { issuer: PublicKey; asset: PublicKey; offeringId: Uint8Array; fiscalPeriod: Uint8Array; termsCommitment: Uint8Array; administrativeOffereeLimit?: number | null; signers?: Signer[] }): Promise<string> {
    assert32(input.offeringId, "offeringId"); assert32(input.termsCommitment, "termsCommitment");
    if (input.fiscalPeriod.length !== 16) throw new Error("fiscalPeriod must be 16 bytes");
    return this.program.methods.createOffering([...input.offeringId], [...input.fiscalPeriod], [...input.termsCommitment], input.administrativeOffereeLimit ?? null)
      .accounts({ issuer: input.issuer, asset: input.asset, offering: this.offeringPda(input.asset, input.offeringId), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async registerOfferee(input: { registrar: PublicKey; asset: PublicKey; offering: PublicKey; offereeRef: Uint8Array; signers?: Signer[] }): Promise<string> {
    assert32(input.offereeRef, "offereeRef");
    const offeree = this.offereePda(input.offering, input.offereeRef);
    return this.program.methods.registerOfferee([...input.offereeRef])
      .accounts({ registrar: input.registrar, asset: input.asset, offering: input.offering,
        authorization: this.registrarAuthorizationPda(input.asset, input.registrar), offeree,
        registrationEvent: this.offereeEventPda(offeree, 0), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async recordOffereeEvent(input: { registrar: PublicKey; asset: PublicKey; offering: PublicKey; offeree: PublicKey; event: OffereeEventName; commitment: Uint8Array; allotmentAmount?: bigint | null; signers?: Signer[] }): Promise<string> {
    assert32(input.commitment, "commitment");
    const state = await this.program.account.offeree.fetch(input.offeree);
    const index = Number(state.eventCount);
    return this.program.methods.recordOffereeEvent({ [input.event]: {} }, [...input.commitment], input.allotmentAmount == null ? null : new BN(input.allotmentAmount.toString()))
      .accounts({ registrar: input.registrar, asset: input.asset, offering: input.offering,
        authorization: this.registrarAuthorizationPda(input.asset, input.registrar), offeree: input.offeree,
        eventRecord: this.offereeEventPda(input.offeree, index), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async closeOffering(input: { issuer: PublicKey; offering: PublicKey; signers?: Signer[] }): Promise<string> {
    return this.program.methods.closeOffering().accounts({ issuer: input.issuer, offering: input.offering })
      .signers(input.signers ?? []).rpc();
  }

  async createDistributionEpoch(input: {
    issuer: PublicKey; asset: PublicKey; epochId: bigint; currencyCode: string; decimals: number; recordDate: bigint;
    snapshotCommitment: Uint8Array; gross: bigint; expensesAndReserves: bigint; distributable: bigint;
    totalWithheld: bigint; totalNet: bigint; merkleRoot: Uint8Array; evidenceManifestCommitment: Uint8Array; signers?: Signer[];
  }): Promise<string> {
    for (const [name, value] of [["snapshotCommitment", input.snapshotCommitment], ["merkleRoot", input.merkleRoot],
      ["evidenceManifestCommitment", input.evidenceManifestCommitment]] as const) assert32(value, name);
    if (!/^[A-Z]{3}$/.test(input.currencyCode)) throw new Error("currencyCode must be three uppercase letters");
    return this.program.methods.createDistributionEpoch(new BN(input.epochId.toString()), [...new TextEncoder().encode(input.currencyCode)], input.decimals,
      new BN(input.recordDate.toString()), [...input.snapshotCommitment], new BN(input.gross.toString()), new BN(input.expensesAndReserves.toString()),
      new BN(input.distributable.toString()), new BN(input.totalWithheld.toString()), new BN(input.totalNet.toString()), [...input.merkleRoot],
      [...input.evidenceManifestCommitment])
      .accounts({ issuer: input.issuer, asset: input.asset, epoch: this.epochPda(input.asset, input.epochId), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  async finalizeDistributionEpoch(input: { issuer: PublicKey; epoch: PublicKey; signers?: Signer[] }): Promise<string> {
    return this.program.methods.finalizeDistributionEpoch().accounts({ issuer: input.issuer, epoch: input.epoch })
      .signers(input.signers ?? []).rpc();
  }

  async recordCorrection(input: {
    issuer: PublicKey; epoch: PublicKey; correctionIndex: number; currencyCode: string; decimals: number;
    gross: bigint; expensesAndReserves: bigint; distributable: bigint; totalWithheld: bigint; totalNet: bigint;
    merkleRoot: Uint8Array; evidenceManifestCommitment: Uint8Array; reasonCommitment: Uint8Array;
    previousCorrection?: PublicKey | null; signers?: Signer[];
  }): Promise<string> {
    assert32(input.merkleRoot, "merkleRoot"); assert32(input.evidenceManifestCommitment, "evidenceManifestCommitment"); assert32(input.reasonCommitment, "reasonCommitment");
    return this.program.methods.recordCorrection(input.correctionIndex, [...new TextEncoder().encode(input.currencyCode)], input.decimals,
      new BN(input.gross.toString()), new BN(input.expensesAndReserves.toString()), new BN(input.distributable.toString()),
      new BN(input.totalWithheld.toString()), new BN(input.totalNet.toString()), [...input.merkleRoot],
      [...input.evidenceManifestCommitment], [...input.reasonCommitment])
      .accounts({ issuer: input.issuer, epoch: input.epoch, previousCorrection: input.previousCorrection ?? null,
        correction: this.correctionPda(input.epoch, input.correctionIndex), systemProgram: SystemProgram.programId })
      .signers(input.signers ?? []).rpc();
  }

  fetchAsset(address: PublicKey): Promise<any> { return this.program.account.assetRegistry.fetch(address); }
  fetchDocumentVersion(address: PublicKey): Promise<any> { return this.program.account.documentVersion.fetch(address); }
  fetchOffering(address: PublicKey): Promise<any> { return this.program.account.offering.fetch(address); }
  fetchOfferee(address: PublicKey): Promise<any> { return this.program.account.offeree.fetch(address); }
  fetchDistributionEpoch(address: PublicKey): Promise<any> { return this.program.account.distributionEpoch.fetch(address); }
}

function pda(programId: PublicKey, seed: string, ...seeds: Uint8Array[]): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(seed), ...seeds.map((part) => Buffer.from(part))], programId)[0];
}

function u64(value: bigint): Uint8Array {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) throw new Error("value must fit in u64");
  const bytes = new Uint8Array(8);
  let current = value;
  for (let i = 0; i < 8; i += 1) { bytes[i] = Number(current & 0xffn); current >>= 8n; }
  return bytes;
}

function u32(value: number): Uint8Array {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new Error("value must fit in u32");
  const bytes = new Uint8Array(4);
  new DataView(bytes.buffer).setUint32(0, value, true);
  return bytes;
}

function assert32(value: Uint8Array, name: string): void {
  if (value.length !== 32) throw new Error(`${name} must be 32 bytes`);
}
