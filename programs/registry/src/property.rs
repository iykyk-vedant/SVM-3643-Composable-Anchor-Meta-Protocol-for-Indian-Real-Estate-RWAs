use anchor_lang::prelude::*;

use crate::ErrorCode;

pub fn create_asset(
    ctx: Context<CreateAsset>,
    asset_id: [u8; 32],
    metadata_commitment: [u8; 32],
    mint: Option<Pubkey>,
) -> Result<()> {
    require!(asset_id != [0; 32], ErrorCode::InvalidIdentifier);
    require!(metadata_commitment != [0; 32], ErrorCode::InvalidCommitment);

    let asset = &mut ctx.accounts.asset;
    asset.issuer = ctx.accounts.issuer.key();
    asset.asset_id = asset_id;
    asset.metadata_commitment = metadata_commitment;
    asset.mint = mint;
    asset.created_at = Clock::get()?.unix_timestamp;
    asset.bump = ctx.bumps.asset;
    Ok(())
}

pub fn add_document_version(
    ctx: Context<AddDocumentVersion>,
    document_key: [u8; 32],
    version: u64,
    document_type: DocumentType,
    document_hash: [u8; 32],
    manifest_hash: [u8; 32],
) -> Result<()> {
    require!(document_key != [0; 32], ErrorCode::InvalidIdentifier);
    require!(document_hash != [0; 32], ErrorCode::InvalidCommitment);
    require!(version > 0, ErrorCode::InvalidVersion);

    let previous = ctx.accounts.previous_document.as_ref();
    match (version, previous) {
        (1, None) => (),
        (next, Some(previous)) => {
            require_keys_eq!(previous.asset, ctx.accounts.asset.key(), ErrorCode::AssetMismatch);
            require!(previous.document_key == document_key, ErrorCode::InvalidPredecessor);
            require!(previous.version.checked_add(1) == Some(next), ErrorCode::InvalidVersion);
            let expected_previous = Pubkey::find_program_address(
                &[
                    b"doc",
                    ctx.accounts.asset.key().as_ref(),
                    document_key.as_ref(),
                    &previous.version.to_le_bytes(),
                ],
                ctx.program_id,
            )
            .0;
            require_keys_eq!(previous.key(), expected_previous, ErrorCode::InvalidPredecessor);
        }
        _ => return err!(ErrorCode::InvalidPredecessor),
    }

    let record = &mut ctx.accounts.document;
    record.asset = ctx.accounts.asset.key();
    record.document_key = document_key;
    record.version = version;
    record.document_type = document_type;
    record.document_hash = document_hash;
    record.manifest_hash = manifest_hash;
    record.previous_version = previous.map(|record| record.key());
    record.issuer = ctx.accounts.issuer.key();
    record.created_at = Clock::get()?.unix_timestamp;
    record.bump = ctx.bumps.document;
    Ok(())
}

pub fn authorize_verifier(ctx: Context<ManageVerifier>, scope: u8) -> Result<()> {
    require!(scope != 0, ErrorCode::InvalidScope);
    let authorization = &mut ctx.accounts.authorization;
    authorization.asset = ctx.accounts.asset.key();
    authorization.verifier = ctx.accounts.verifier.key();
    authorization.scope = scope;
    authorization.active = true;
    authorization.bump = ctx.bumps.authorization;
    Ok(())
}

pub fn revoke_verifier(ctx: Context<ManageVerifier>) -> Result<()> {
    require!(ctx.accounts.authorization.active, ErrorCode::AlreadyRevoked);
    ctx.accounts.authorization.active = false;
    Ok(())
}

pub fn attest_document(
    ctx: Context<AttestDocument>,
    review_commitment: [u8; 32],
    outcome: AttestationOutcome,
) -> Result<()> {
    require!(ctx.accounts.authorization.active, ErrorCode::Unauthorized);
    require!(review_commitment != [0; 32], ErrorCode::InvalidCommitment);
    let type_scope = 1u8 << (ctx.accounts.document.document_type as u8);
    require!(ctx.accounts.authorization.scope & type_scope != 0, ErrorCode::InvalidScope);

    let attestation = &mut ctx.accounts.attestation;
    attestation.document = ctx.accounts.document.key();
    attestation.verifier = ctx.accounts.verifier.key();
    attestation.review_commitment = review_commitment;
    attestation.outcome = outcome;
    attestation.created_at = Clock::get()?.unix_timestamp;
    attestation.bump = ctx.bumps.attestation;
    Ok(())
}

#[derive(Accounts)]
#[instruction(asset_id: [u8; 32])]
pub struct CreateAsset<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(
        init,
        payer = issuer,
        space = 8 + AssetRegistry::INIT_SPACE,
        seeds = [b"asset", issuer.key().as_ref(), asset_id.as_ref()],
        bump
    )]
    pub asset: Account<'info, AssetRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(document_key: [u8; 32], version: u64)]
pub struct AddDocumentVersion<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    #[account(
        seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        init,
        payer = issuer,
        space = 8 + DocumentVersion::INIT_SPACE,
        seeds = [b"doc", asset.key().as_ref(), document_key.as_ref(), &version.to_le_bytes()],
        bump
    )]
    pub document: Account<'info, DocumentVersion>,
    pub previous_document: Option<Account<'info, DocumentVersion>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageVerifier<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    /// CHECK: This pubkey is only used as a PDA seed; the verifier must sign when attesting.
    pub verifier: UncheckedAccount<'info>,
    #[account(
        seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        init_if_needed,
        payer = issuer,
        space = 8 + VerifierAuthorization::INIT_SPACE,
        seeds = [b"verifier", asset.key().as_ref(), verifier.key().as_ref()],
        bump
    )]
    pub authorization: Account<'info, VerifierAuthorization>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AttestDocument<'info> {
    #[account(mut)]
    pub verifier: Signer<'info>,
    #[account(
        seeds = [b"verifier", document.asset.as_ref(), verifier.key().as_ref()],
        bump = authorization.bump,
        has_one = verifier @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch
    )]
    pub authorization: Account<'info, VerifierAuthorization>,
    #[account(
        seeds = [b"asset", asset.issuer.as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump
    )]
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        seeds = [b"doc", asset.key().as_ref(), document.document_key.as_ref(), &document.version.to_le_bytes()],
        bump = document.bump
    )]
    pub document: Account<'info, DocumentVersion>,
    #[account(
        init,
        payer = verifier,
        space = 8 + VerifierAttestation::INIT_SPACE,
        seeds = [b"attestation", document.key().as_ref(), verifier.key().as_ref()],
        bump
    )]
    pub attestation: Account<'info, VerifierAttestation>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct AssetRegistry {
    pub issuer: Pubkey,
    pub asset_id: [u8; 32],
    pub metadata_commitment: [u8; 32],
    pub mint: Option<Pubkey>,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct DocumentVersion {
    pub asset: Pubkey,
    pub document_key: [u8; 32],
    pub version: u64,
    pub document_type: DocumentType,
    pub document_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub previous_version: Option<Pubkey>,
    pub issuer: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VerifierAuthorization {
    pub asset: Pubkey,
    pub verifier: Pubkey,
    pub scope: u8,
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VerifierAttestation {
    pub document: Pubkey,
    pub verifier: Pubkey,
    pub review_commitment: [u8; 32],
    pub outcome: AttestationOutcome,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum AttestationOutcome {
    Reviewed,
    NeedsCorrection,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum DocumentType {
    TitleSearch,
    Valuation,
    SpvEvidence,
    ReraRecord,
    LeaseStatement,
    Other,
}
