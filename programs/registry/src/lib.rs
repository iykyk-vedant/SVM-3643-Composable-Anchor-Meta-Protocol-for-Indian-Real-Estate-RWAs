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
}
