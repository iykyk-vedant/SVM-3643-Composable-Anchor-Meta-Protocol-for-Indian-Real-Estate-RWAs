use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWxTWqkZ6W2BeZ7FEfcYkgMQhgZ");

pub mod distribution;
pub mod placement;
pub mod property;

#[program]
pub mod indian_rwa_registry {
    use super::*;

    pub fn create_asset(
        ctx: Context<property::CreateAsset>,
        asset_id: [u8; 32],
        metadata_commitment: [u8; 32],
        mint: Option<Pubkey>,
    ) -> Result<()> {
        property::create_asset(ctx, asset_id, metadata_commitment, mint)
    }

    pub fn add_document_version(
        ctx: Context<property::AddDocumentVersion>,
        document_key: [u8; 32],
        version: u64,
        document_type: property::DocumentType,
        document_hash: [u8; 32],
        manifest_hash: [u8; 32],
    ) -> Result<()> {
        property::add_document_version(
            ctx,
            document_key,
            version,
            document_type,
            document_hash,
            manifest_hash,
        )
    }

    pub fn authorize_verifier(ctx: Context<property::ManageVerifier>, scope: u8) -> Result<()> {
        property::authorize_verifier(ctx, scope)
    }

    pub fn revoke_verifier(ctx: Context<property::ManageVerifier>) -> Result<()> {
        property::revoke_verifier(ctx)
    }

    pub fn attest_document(
        ctx: Context<property::AttestDocument>,
        review_commitment: [u8; 32],
        outcome: property::AttestationOutcome,
    ) -> Result<()> {
        property::attest_document(ctx, review_commitment, outcome)
    }

    pub fn authorize_registrar(ctx: Context<placement::ManageRegistrar>) -> Result<()> {
        placement::authorize_registrar(ctx)
    }

    pub fn revoke_registrar(ctx: Context<placement::ManageRegistrar>) -> Result<()> {
        placement::revoke_registrar(ctx)
    }

    pub fn create_offering(
        ctx: Context<placement::CreateOffering>,
        offering_id: [u8; 32],
        fiscal_period: [u8; 16],
        terms_commitment: [u8; 32],
        administrative_offeree_limit: Option<u32>,
    ) -> Result<()> {
        placement::create_offering(ctx, offering_id, fiscal_period, terms_commitment, administrative_offeree_limit)
    }

    pub fn register_offeree(ctx: Context<placement::RegisterOfferee>, offeree_ref: [u8; 32]) -> Result<()> {
        placement::register_offeree(ctx, offeree_ref)
    }

    pub fn record_offeree_event(
        ctx: Context<placement::RecordOffereeEvent>,
        event: placement::OffereeEvent,
        commitment: [u8; 32],
        allotment_amount: Option<u64>,
    ) -> Result<()> {
        placement::record_offeree_event(ctx, event, commitment, allotment_amount)
    }

    pub fn close_offering(ctx: Context<placement::CloseOffering>) -> Result<()> {
        placement::close_offering(ctx)
    }

    pub fn create_distribution_epoch(
        ctx: Context<distribution::CreateEpoch>,
        epoch_id: u64,
        currency_code: [u8; 3],
        decimals: u8,
        record_date: i64,
        snapshot_commitment: [u8; 32],
        gross: u64,
        expenses_and_reserves: u64,
        distributable: u64,
        total_withheld: u64,
        total_net: u64,
        merkle_root: [u8; 32],
        evidence_manifest_commitment: [u8; 32],
    ) -> Result<()> {
        distribution::create_epoch(ctx, epoch_id, currency_code, decimals, record_date, snapshot_commitment,
            gross, expenses_and_reserves, distributable, total_withheld, total_net, merkle_root,
            evidence_manifest_commitment)
    }

    pub fn finalize_distribution_epoch(ctx: Context<distribution::FinalizeEpoch>) -> Result<()> {
        distribution::finalize_epoch(ctx)
    }

    pub fn record_correction(
        ctx: Context<distribution::RecordCorrection>,
        correction_index: u32,
        currency_code: [u8; 3],
        decimals: u8,
        gross: u64,
        expenses_and_reserves: u64,
        distributable: u64,
        total_withheld: u64,
        total_net: u64,
        merkle_root: [u8; 32],
        evidence_manifest_commitment: [u8; 32],
        reason_commitment: [u8; 32],
    ) -> Result<()> {
        distribution::record_correction(ctx, correction_index, currency_code, decimals, gross,
            expenses_and_reserves, distributable, total_withheld, total_net, merkle_root,
            evidence_manifest_commitment, reason_commitment)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("The signer is not authorized for this asset")]
    Unauthorized,
    #[msg("Identifier must be a non-zero opaque 32-byte value")]
    InvalidIdentifier,
    #[msg("Commitment must be non-zero")]
    InvalidCommitment,
    #[msg("Document version must be positive and sequential")]
    InvalidVersion,
    #[msg("Document predecessor does not match the same asset and document key")]
    InvalidPredecessor,
    #[msg("Referenced account belongs to another asset")]
    AssetMismatch,
    #[msg("Verifier scope is empty or does not include this document category")]
    InvalidScope,
    #[msg("Authorization is already revoked")]
    AlreadyRevoked,
    #[msg("Offering is closed")]
    OfferingClosed,
    #[msg("Issuer-defined administrative offeree limit reached")]
    AdministrativeLimitReached,
    #[msg("Checked arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Offeree event is not valid for the current state")]
    InvalidStateTransition,
    #[msg("Amount or optional amount is invalid")]
    InvalidAmount,
    #[msg("Distribution arithmetic or supplied totals are inconsistent")]
    InvalidDistributionTotals,
    #[msg("Distribution epoch is already finalized")]
    EpochFinalized,
    #[msg("A correction must append to the prior correction in order")]
    InvalidCorrectionChain,
    #[msg("Currency must be a three-letter uppercase code")]
    InvalidCurrency,
    #[msg("Distribution epoch must be finalized first")]
    EpochNotFinalized,
}
