use anchor_lang::prelude::*;

declare_id!("Bt9HpxHpiTwTg4iJpEn3hKKktakc6scc1hZNAdLKYwQs");

pub mod distribution;
pub mod placement;
pub mod property;

#[derive(Accounts)]
#[instruction(asset_id: [u8; 32])]
pub struct CreateAsset<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(init, payer = issuer, space = 8 + property::AssetRegistry::INIT_SPACE,
        seeds = [b"asset", issuer.key().as_ref(), asset_id.as_ref()], bump)]
    pub asset: Account<'info, property::AssetRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(document_key: [u8; 32], version: u64)]
pub struct AddDocumentVersion<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(init, payer = issuer, space = 8 + property::DocumentVersion::INIT_SPACE,
        seeds = [b"doc", asset.key().as_ref(), document_key.as_ref(), &version.to_le_bytes()], bump)]
    pub document: Account<'info, property::DocumentVersion>,
    pub previous_document: Option<Account<'info, property::DocumentVersion>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageVerifier<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    /// CHECK: Used as the verifier authority PDA seed; verifier signs attestation transactions.
    pub verifier: UncheckedAccount<'info>,
    #[account(seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(init_if_needed, payer = issuer, space = 8 + property::VerifierAuthorization::INIT_SPACE,
        seeds = [b"verifier", asset.key().as_ref(), verifier.key().as_ref()], bump)]
    pub authorization: Account<'info, property::VerifierAuthorization>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AttestDocument<'info> {
    #[account(mut)]
    pub verifier: Signer<'info>,
    #[account(seeds = [b"verifier", document.asset.as_ref(), verifier.key().as_ref()],
        bump = authorization.bump, has_one = verifier @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch)]
    pub authorization: Account<'info, property::VerifierAuthorization>,
    #[account(seeds = [b"asset", asset.issuer.as_ref(), asset.asset_id.as_ref()], bump = asset.bump)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(seeds = [b"doc", asset.key().as_ref(), document.document_key.as_ref(), &document.version.to_le_bytes()],
        bump = document.bump)]
    pub document: Account<'info, property::DocumentVersion>,
    #[account(init, payer = verifier, space = 8 + property::VerifierAttestation::INIT_SPACE,
        seeds = [b"attestation", document.key().as_ref(), verifier.key().as_ref()], bump)]
    pub attestation: Account<'info, property::VerifierAttestation>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageRegistrar<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    /// CHECK: Used only as the delegated authority PDA seed.
    pub registrar: UncheckedAccount<'info>,
    #[account(seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(init_if_needed, payer = issuer, space = 8 + placement::RegistrarAuthorization::INIT_SPACE,
        seeds = [b"registrar", asset.key().as_ref(), registrar.key().as_ref()], bump)]
    pub authorization: Account<'info, placement::RegistrarAuthorization>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(offering_id: [u8; 32])]
pub struct CreateOffering<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(init, payer = issuer, space = 8 + placement::Offering::INIT_SPACE,
        seeds = [b"offering", asset.key().as_ref(), offering_id.as_ref()], bump)]
    pub offering: Account<'info, placement::Offering>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(offeree_ref: [u8; 32])]
pub struct RegisterOfferee<'info> {
    #[account(mut)]
    pub registrar: Signer<'info>,
    #[account(mut, seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump, has_one = asset @ ErrorCode::AssetMismatch)]
    pub offering: Account<'info, placement::Offering>,
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(seeds = [b"registrar", offering.asset.as_ref(), registrar.key().as_ref()],
        bump = authorization.bump, has_one = registrar @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch, constraint = authorization.active @ ErrorCode::Unauthorized)]
    pub authorization: Account<'info, placement::RegistrarAuthorization>,
    #[account(init, payer = registrar, space = 8 + placement::Offeree::INIT_SPACE,
        seeds = [b"offeree", offering.key().as_ref(), offeree_ref.as_ref()], bump)]
    pub offeree: Account<'info, placement::Offeree>,
    #[account(init, payer = registrar, space = 8 + placement::OffereeEventRecord::INIT_SPACE,
        seeds = [b"offeree-event", offeree.key().as_ref(), &0u32.to_le_bytes()], bump)]
    pub registration_event: Account<'info, placement::OffereeEventRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordOffereeEvent<'info> {
    #[account(mut)]
    pub registrar: Signer<'info>,
    #[account(mut, seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump, has_one = asset @ ErrorCode::AssetMismatch)]
    pub offering: Account<'info, placement::Offering>,
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(seeds = [b"registrar", offering.asset.as_ref(), registrar.key().as_ref()],
        bump = authorization.bump, has_one = registrar @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch, constraint = authorization.active @ ErrorCode::Unauthorized)]
    pub authorization: Account<'info, placement::RegistrarAuthorization>,
    #[account(mut, seeds = [b"offeree", offering.key().as_ref(), offeree.offeree_ref.as_ref()],
        bump = offeree.bump, has_one = offering @ ErrorCode::AssetMismatch)]
    pub offeree: Account<'info, placement::Offeree>,
    #[account(init, payer = registrar, space = 8 + placement::OffereeEventRecord::INIT_SPACE,
        seeds = [b"offeree-event", offeree.key().as_ref(), &offeree.event_count.to_le_bytes()], bump)]
    pub event_record: Account<'info, placement::OffereeEventRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseOffering<'info> {
    pub issuer: Signer<'info>,
    #[account(mut, seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub offering: Account<'info, placement::Offering>,
}

#[derive(Accounts)]
#[instruction(epoch_id: u64)]
pub struct CreateEpoch<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub asset: Account<'info, property::AssetRegistry>,
    #[account(init, payer = issuer, space = 8 + distribution::DistributionEpoch::INIT_SPACE,
        seeds = [b"epoch", asset.key().as_ref(), &epoch_id.to_le_bytes()], bump)]
    pub epoch: Account<'info, distribution::DistributionEpoch>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeEpoch<'info> {
    pub issuer: Signer<'info>,
    #[account(mut, seeds = [b"epoch", epoch.asset.as_ref(), &epoch.epoch_id.to_le_bytes()],
        bump = epoch.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub epoch: Account<'info, distribution::DistributionEpoch>,
}

#[derive(Accounts)]
#[instruction(correction_index: u32)]
pub struct RecordCorrection<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(seeds = [b"epoch", epoch.asset.as_ref(), &epoch.epoch_id.to_le_bytes()],
        bump = epoch.bump, has_one = issuer @ ErrorCode::Unauthorized)]
    pub epoch: Account<'info, distribution::DistributionEpoch>,
    pub previous_correction: Option<Account<'info, distribution::CorrectionEpoch>>,
    #[account(init, payer = issuer, space = 8 + distribution::CorrectionEpoch::INIT_SPACE,
        seeds = [b"correction", epoch.key().as_ref(), &correction_index.to_le_bytes()], bump)]
    pub correction: Account<'info, distribution::CorrectionEpoch>,
    pub system_program: Program<'info, System>,
}

#[program]
pub mod indian_rwa_registry {
    use super::*;

    pub fn create_asset(
        ctx: Context<CreateAsset>,
        asset_id: [u8; 32],
        metadata_commitment: [u8; 32],
        mint: Option<Pubkey>,
    ) -> Result<()> {
        property::create_asset(ctx, asset_id, metadata_commitment, mint)
    }

    pub fn add_document_version(
        ctx: Context<AddDocumentVersion>,
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

    pub fn authorize_verifier(ctx: Context<ManageVerifier>, scope: u8) -> Result<()> {
        property::authorize_verifier(ctx, scope)
    }

    pub fn revoke_verifier(ctx: Context<ManageVerifier>) -> Result<()> {
        property::revoke_verifier(ctx)
    }

    pub fn attest_document(
        ctx: Context<AttestDocument>,
        review_commitment: [u8; 32],
        outcome: property::AttestationOutcome,
    ) -> Result<()> {
        property::attest_document(ctx, review_commitment, outcome)
    }

    pub fn authorize_registrar(ctx: Context<ManageRegistrar>) -> Result<()> {
        placement::authorize_registrar(ctx)
    }

    pub fn revoke_registrar(ctx: Context<ManageRegistrar>) -> Result<()> {
        placement::revoke_registrar(ctx)
    }

    pub fn create_offering(
        ctx: Context<CreateOffering>,
        offering_id: [u8; 32],
        fiscal_period: [u8; 16],
        terms_commitment: [u8; 32],
        administrative_offeree_limit: Option<u32>,
    ) -> Result<()> {
        placement::create_offering(
            ctx,
            offering_id,
            fiscal_period,
            terms_commitment,
            administrative_offeree_limit,
        )
    }

    pub fn register_offeree(ctx: Context<RegisterOfferee>, offeree_ref: [u8; 32]) -> Result<()> {
        placement::register_offeree(ctx, offeree_ref)
    }

    pub fn record_offeree_event(
        ctx: Context<RecordOffereeEvent>,
        event: placement::OffereeEvent,
        commitment: [u8; 32],
        allotment_amount: Option<u64>,
    ) -> Result<()> {
        placement::record_offeree_event(ctx, event, commitment, allotment_amount)
    }

    pub fn close_offering(ctx: Context<CloseOffering>) -> Result<()> {
        placement::close_offering(ctx)
    }

    pub fn create_distribution_epoch(
        ctx: Context<CreateEpoch>,
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
        distribution::create_epoch(
            ctx,
            epoch_id,
            currency_code,
            decimals,
            record_date,
            snapshot_commitment,
            gross,
            expenses_and_reserves,
            distributable,
            total_withheld,
            total_net,
            merkle_root,
            evidence_manifest_commitment,
        )
    }

    pub fn finalize_distribution_epoch(ctx: Context<FinalizeEpoch>) -> Result<()> {
        distribution::finalize_epoch(ctx)
    }

    pub fn record_correction(
        ctx: Context<RecordCorrection>,
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
        distribution::record_correction(
            ctx,
            correction_index,
            currency_code,
            decimals,
            gross,
            expenses_and_reserves,
            distributable,
            total_withheld,
            total_net,
            merkle_root,
            evidence_manifest_commitment,
            reason_commitment,
        )
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
    #[msg("Currency code or decimal precision is invalid for this record")]
    InvalidCurrency,
    #[msg("Distribution epoch must be finalized first")]
    EpochNotFinalized,
}
