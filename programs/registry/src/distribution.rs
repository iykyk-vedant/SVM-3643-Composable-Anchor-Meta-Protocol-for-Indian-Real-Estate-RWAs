use anchor_lang::prelude::*;

use crate::{property::AssetRegistry, ErrorCode};

pub fn create_epoch(
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
    validate_distribution(gross, expenses_and_reserves, distributable, total_withheld, total_net)?;
    require!(valid_currency(currency_code), ErrorCode::InvalidCurrency);
    require!(snapshot_commitment != [0; 32], ErrorCode::InvalidCommitment);
    require!(merkle_root != [0; 32], ErrorCode::InvalidCommitment);
    require!(evidence_manifest_commitment != [0; 32], ErrorCode::InvalidCommitment);

    let epoch = &mut ctx.accounts.epoch;
    epoch.asset = ctx.accounts.asset.key();
    epoch.issuer = ctx.accounts.issuer.key();
    epoch.epoch_id = epoch_id;
    epoch.currency_code = currency_code;
    epoch.decimals = decimals;
    epoch.record_date = record_date;
    epoch.snapshot_commitment = snapshot_commitment;
    epoch.gross = gross;
    epoch.expenses_and_reserves = expenses_and_reserves;
    epoch.distributable = distributable;
    epoch.total_withheld = total_withheld;
    epoch.total_net = total_net;
    epoch.merkle_root = merkle_root;
    epoch.evidence_manifest_commitment = evidence_manifest_commitment;
    epoch.finalized = false;
    epoch.created_at = Clock::get()?.unix_timestamp;
    epoch.bump = ctx.bumps.epoch;
    emit!(DistributionEpochCreated { asset: epoch.asset, epoch: epoch.key(), issuer: epoch.issuer });
    Ok(())
}

pub fn finalize_epoch(ctx: Context<FinalizeEpoch>) -> Result<()> {
    require!(!ctx.accounts.epoch.finalized, ErrorCode::EpochFinalized);
    ctx.accounts.epoch.finalized = true;
    ctx.accounts.epoch.finalized_at = Some(Clock::get()?.unix_timestamp);
    emit!(DistributionEpochFinalized { epoch: ctx.accounts.epoch.key() });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    require!(ctx.accounts.epoch.finalized, ErrorCode::EpochNotFinalized);
    validate_distribution(gross, expenses_and_reserves, distributable, total_withheld, total_net)?;
    require!(valid_currency(currency_code), ErrorCode::InvalidCurrency);
    require!(currency_code == ctx.accounts.epoch.currency_code, ErrorCode::InvalidCurrency);
    require!(decimals == ctx.accounts.epoch.decimals, ErrorCode::InvalidCurrency);
    require!(merkle_root != [0; 32], ErrorCode::InvalidCommitment);
    require!(evidence_manifest_commitment != [0; 32], ErrorCode::InvalidCommitment);
    require!(reason_commitment != [0; 32], ErrorCode::InvalidCommitment);
    match (correction_index, ctx.accounts.previous_correction.as_ref()) {
        (0, None) => (),
        (index, Some(previous)) => {
            require_keys_eq!(previous.epoch, ctx.accounts.epoch.key(), ErrorCode::InvalidCorrectionChain);
            require!(previous.correction_index.checked_add(1) == Some(index), ErrorCode::InvalidCorrectionChain);
            let expected = Pubkey::find_program_address(
                &[b"correction", ctx.accounts.epoch.key().as_ref(), &previous.correction_index.to_le_bytes()],
                ctx.program_id,
            ).0;
            require_keys_eq!(previous.key(), expected, ErrorCode::InvalidCorrectionChain);
        }
        _ => return err!(ErrorCode::InvalidCorrectionChain),
    }
    let correction = &mut ctx.accounts.correction;
    correction.epoch = ctx.accounts.epoch.key();
    correction.previous_correction = ctx.accounts.previous_correction.as_ref().map(|account| account.key());
    correction.correction_index = correction_index;
    correction.currency_code = currency_code;
    correction.decimals = decimals;
    correction.gross = gross;
    correction.expenses_and_reserves = expenses_and_reserves;
    correction.distributable = distributable;
    correction.total_withheld = total_withheld;
    correction.total_net = total_net;
    correction.merkle_root = merkle_root;
    correction.evidence_manifest_commitment = evidence_manifest_commitment;
    correction.reason_commitment = reason_commitment;
    correction.created_at = Clock::get()?.unix_timestamp;
    correction.bump = ctx.bumps.correction;
    emit!(DistributionCorrectionRecorded { epoch: correction.epoch, correction: correction.key(), index: correction_index });
    Ok(())
}

pub fn validate_distribution(gross: u64, expenses: u64, distributable: u64, withheld: u64, net: u64) -> Result<()> {
    require!(gross.checked_sub(expenses) == Some(distributable), ErrorCode::InvalidDistributionTotals);
    require!(distributable.checked_sub(withheld) == Some(net), ErrorCode::InvalidDistributionTotals);
    Ok(())
}

fn valid_currency(currency: [u8; 3]) -> bool {
    currency.iter().all(u8::is_ascii_uppercase)
}

#[derive(Accounts)]
#[instruction(epoch_id: u64)]
pub struct CreateEpoch<'info> {
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
        space = 8 + DistributionEpoch::INIT_SPACE,
        seeds = [b"epoch", asset.key().as_ref(), &epoch_id.to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, DistributionEpoch>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeEpoch<'info> {
    pub issuer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.asset.as_ref(), &epoch.epoch_id.to_le_bytes()],
        bump = epoch.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub epoch: Account<'info, DistributionEpoch>,
}

#[derive(Accounts)]
#[instruction(correction_index: u32)]
pub struct RecordCorrection<'info> {
    pub issuer: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.asset.as_ref(), &epoch.epoch_id.to_le_bytes()],
        bump = epoch.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub epoch: Account<'info, DistributionEpoch>,
    pub previous_correction: Option<Account<'info, CorrectionEpoch>>,
    #[account(
        init,
        payer = issuer,
        space = 8 + CorrectionEpoch::INIT_SPACE,
        seeds = [b"correction", epoch.key().as_ref(), &correction_index.to_le_bytes()],
        bump
    )]
    pub correction: Account<'info, CorrectionEpoch>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct DistributionEpoch {
    pub asset: Pubkey,
    pub issuer: Pubkey,
    pub epoch_id: u64,
    pub currency_code: [u8; 3],
    pub decimals: u8,
    pub record_date: i64,
    pub snapshot_commitment: [u8; 32],
    pub gross: u64,
    pub expenses_and_reserves: u64,
    pub distributable: u64,
    pub total_withheld: u64,
    pub total_net: u64,
    pub merkle_root: [u8; 32],
    pub evidence_manifest_commitment: [u8; 32],
    pub finalized: bool,
    pub created_at: i64,
    pub finalized_at: Option<i64>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CorrectionEpoch {
    pub epoch: Pubkey,
    pub previous_correction: Option<Pubkey>,
    pub correction_index: u32,
    pub currency_code: [u8; 3],
    pub decimals: u8,
    pub gross: u64,
    pub expenses_and_reserves: u64,
    pub distributable: u64,
    pub total_withheld: u64,
    pub total_net: u64,
    pub merkle_root: [u8; 32],
    pub evidence_manifest_commitment: [u8; 32],
    pub reason_commitment: [u8; 32],
    pub created_at: i64,
    pub bump: u8,
}

#[event]
pub struct DistributionEpochCreated { pub asset: Pubkey, pub epoch: Pubkey, pub issuer: Pubkey }
#[event]
pub struct DistributionEpochFinalized { pub epoch: Pubkey }
#[event]
pub struct DistributionCorrectionRecorded { pub epoch: Pubkey, pub correction: Pubkey, pub index: u32 }

#[cfg(test)]
mod tests {
    use super::validate_distribution;

    #[test]
    fn rejects_withheld_exceeding_distributable() {
        assert!(validate_distribution(100, 20, 80, 81, 0).is_err());
    }

    #[test]
    fn rejects_inconsistent_totals_and_accepts_balanced_totals() {
        assert!(validate_distribution(100, 20, 79, 10, 69).is_err());
        assert!(validate_distribution(100, 20, 80, 10, 70).is_ok());
    }
}
