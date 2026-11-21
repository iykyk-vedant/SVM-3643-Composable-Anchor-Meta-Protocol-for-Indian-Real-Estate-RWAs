use anchor_lang::prelude::*;

use crate::{property::AssetRegistry, ErrorCode};

pub fn authorize_registrar(ctx: Context<ManageRegistrar>) -> Result<()> {
    let role = &mut ctx.accounts.authorization;
    role.asset = ctx.accounts.asset.key();
    role.registrar = ctx.accounts.registrar.key();
    role.active = true;
    role.bump = ctx.bumps.authorization;
    emit!(RegistrarAuthorizationChanged { asset: role.asset, registrar: role.registrar, active: true });
    Ok(())
}

pub fn revoke_registrar(ctx: Context<ManageRegistrar>) -> Result<()> {
    require!(ctx.accounts.authorization.active, ErrorCode::AlreadyRevoked);
    ctx.accounts.authorization.active = false;
    emit!(RegistrarAuthorizationChanged {
        asset: ctx.accounts.authorization.asset,
        registrar: ctx.accounts.authorization.registrar,
        active: false,
    });
    Ok(())
}

pub fn create_offering(
    ctx: Context<CreateOffering>,
    offering_id: [u8; 32],
    fiscal_period: [u8; 16],
    terms_commitment: [u8; 32],
    administrative_offeree_limit: Option<u32>,
) -> Result<()> {
    require!(offering_id != [0; 32], ErrorCode::InvalidIdentifier);
    require!(terms_commitment != [0; 32], ErrorCode::InvalidCommitment);
    let offering = &mut ctx.accounts.offering;
    offering.asset = ctx.accounts.asset.key();
    offering.issuer = ctx.accounts.issuer.key();
    offering.offering_id = offering_id;
    offering.fiscal_period = fiscal_period;
    offering.terms_commitment = terms_commitment;
    offering.administrative_offeree_limit = administrative_offeree_limit;
    offering.registered_count = 0;
    offering.invited_count = 0;
    offering.applied_count = 0;
    offering.allotted_count = 0;
    offering.closed = false;
    offering.created_at = Clock::get()?.unix_timestamp;
    offering.bump = ctx.bumps.offering;
    emit!(OfferingCreated { asset: offering.asset, offering: offering.key(), issuer: offering.issuer });
    Ok(())
}

pub fn register_offeree(ctx: Context<RegisterOfferee>, offeree_ref: [u8; 32]) -> Result<()> {
    require!(offeree_ref != [0; 32], ErrorCode::InvalidIdentifier);
    require!(!ctx.accounts.offering.closed, ErrorCode::OfferingClosed);
    let offering = &mut ctx.accounts.offering;
    if let Some(limit) = offering.administrative_offeree_limit {
        require!(offering.registered_count < limit, ErrorCode::AdministrativeLimitReached);
    }
    offering.registered_count = offering.registered_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
    let offeree = &mut ctx.accounts.offeree;
    offeree.offering = offering.key();
    offeree.offeree_ref = offeree_ref;
    offeree.state = OffereeState::Registered;
    offeree.invitation_commitment = None;
    offeree.application_commitment = None;
    offeree.allotment_commitment = None;
    offeree.allotment_amount = None;
    offeree.event_count = 1;
    offeree.created_at = Clock::get()?.unix_timestamp;
    offeree.bump = ctx.bumps.offeree;
    let first_event = &mut ctx.accounts.registration_event;
    first_event.offeree = offeree.key();
    first_event.index = 0;
    first_event.event = OffereeEvent::Registered;
    first_event.commitment = [0; 32];
    first_event.amount = None;
    first_event.created_at = offeree.created_at;
    first_event.bump = ctx.bumps.registration_event;
    emit!(OffereeRegistered { offering: offering.key(), offeree_ref });
    Ok(())
}

pub fn record_offeree_event(
    ctx: Context<RecordOffereeEvent>,
    event: OffereeEvent,
    commitment: [u8; 32],
    allotment_amount: Option<u64>,
) -> Result<()> {
    require!(!ctx.accounts.offering.closed, ErrorCode::OfferingClosed);
    require!(commitment != [0; 32], ErrorCode::InvalidCommitment);
    let offeree = &mut ctx.accounts.offeree;
    let offering = &mut ctx.accounts.offering;
    match event {
        OffereeEvent::Registered => return err!(ErrorCode::InvalidStateTransition),
        OffereeEvent::Invite => {
            require!(offeree.state == OffereeState::Registered, ErrorCode::InvalidStateTransition);
            offeree.state = OffereeState::Invited;
            offeree.invitation_commitment = Some(commitment);
            offering.invited_count = offering.invited_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        OffereeEvent::Apply => {
            require!(offeree.state == OffereeState::Invited, ErrorCode::InvalidStateTransition);
            offeree.state = OffereeState::Applied;
            offeree.application_commitment = Some(commitment);
            offering.applied_count = offering.applied_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        OffereeEvent::Allot => {
            require!(offeree.state == OffereeState::Applied, ErrorCode::InvalidStateTransition);
            let amount = allotment_amount.ok_or(ErrorCode::InvalidAmount)?;
            require!(amount > 0, ErrorCode::InvalidAmount);
            offeree.state = OffereeState::Allotted;
            offeree.allotment_commitment = Some(commitment);
            offeree.allotment_amount = Some(amount);
            offering.allotted_count = offering.allotted_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        OffereeEvent::Reject => {
            require!(offeree.state == OffereeState::Applied, ErrorCode::InvalidStateTransition);
            require!(allotment_amount.is_none(), ErrorCode::InvalidAmount);
            offeree.state = OffereeState::Rejected;
        }
        OffereeEvent::Withdraw => {
            require!(
                matches!(offeree.state, OffereeState::Registered | OffereeState::Invited | OffereeState::Applied),
                ErrorCode::InvalidStateTransition
            );
            require!(allotment_amount.is_none(), ErrorCode::InvalidAmount);
            offeree.state = OffereeState::Withdrawn;
        }
    }
    let event_record = &mut ctx.accounts.event_record;
    event_record.offeree = offeree.key();
    event_record.index = offeree.event_count;
    event_record.event = event;
    event_record.commitment = commitment;
    event_record.amount = allotment_amount;
    event_record.created_at = Clock::get()?.unix_timestamp;
    event_record.bump = ctx.bumps.event_record;
    offeree.event_count = offeree.event_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
    emit!(OffereeEventRecorded { offering: offering.key(), offeree_ref: offeree.offeree_ref, event });
    Ok(())
}

pub fn close_offering(ctx: Context<CloseOffering>) -> Result<()> {
    require!(!ctx.accounts.offering.closed, ErrorCode::OfferingClosed);
    ctx.accounts.offering.closed = true;
    emit!(OfferingClosedEvent { offering: ctx.accounts.offering.key() });
    Ok(())
}

#[derive(Accounts)]
pub struct ManageRegistrar<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,
    /// CHECK: The registrar pubkey is used only as the delegated authority PDA seed.
    pub registrar: UncheckedAccount<'info>,
    #[account(
        seeds = [b"asset", issuer.key().as_ref(), asset.asset_id.as_ref()],
        bump = asset.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        init_if_needed,
        payer = issuer,
        space = 8 + RegistrarAuthorization::INIT_SPACE,
        seeds = [b"registrar", asset.key().as_ref(), registrar.key().as_ref()],
        bump
    )]
    pub authorization: Account<'info, RegistrarAuthorization>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(offering_id: [u8; 32])]
pub struct CreateOffering<'info> {
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
        space = 8 + Offering::INIT_SPACE,
        seeds = [b"offering", asset.key().as_ref(), offering_id.as_ref()],
        bump
    )]
    pub offering: Account<'info, Offering>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(offeree_ref: [u8; 32])]
pub struct RegisterOfferee<'info> {
    #[account(mut)]
    pub registrar: Signer<'info>,
    #[account(
        mut,
        seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump,
        has_one = asset @ ErrorCode::AssetMismatch
    )]
    pub offering: Account<'info, Offering>,
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        seeds = [b"registrar", offering.asset.as_ref(), registrar.key().as_ref()],
        bump = authorization.bump,
        has_one = registrar @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch,
        constraint = authorization.active @ ErrorCode::Unauthorized
    )]
    pub authorization: Account<'info, RegistrarAuthorization>,
    #[account(
        init,
        payer = registrar,
        space = 8 + Offeree::INIT_SPACE,
        seeds = [b"offeree", offering.key().as_ref(), offeree_ref.as_ref()],
        bump
    )]
    pub offeree: Account<'info, Offeree>,
    #[account(
        init,
        payer = registrar,
        space = 8 + OffereeEventRecord::INIT_SPACE,
        seeds = [b"offeree-event", offeree.key().as_ref(), &0u32.to_le_bytes()],
        bump
    )]
    pub registration_event: Account<'info, OffereeEventRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordOffereeEvent<'info> {
    pub registrar: Signer<'info>,
    #[account(
        mut,
        seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump,
        has_one = asset @ ErrorCode::AssetMismatch
    )]
    pub offering: Account<'info, Offering>,
    pub asset: Account<'info, AssetRegistry>,
    #[account(
        seeds = [b"registrar", offering.asset.as_ref(), registrar.key().as_ref()],
        bump = authorization.bump,
        has_one = registrar @ ErrorCode::Unauthorized,
        has_one = asset @ ErrorCode::AssetMismatch,
        constraint = authorization.active @ ErrorCode::Unauthorized
    )]
    pub authorization: Account<'info, RegistrarAuthorization>,
    #[account(
        mut,
        seeds = [b"offeree", offering.key().as_ref(), offeree.offeree_ref.as_ref()],
        bump = offeree.bump,
        has_one = offering @ ErrorCode::AssetMismatch
    )]
    pub offeree: Account<'info, Offeree>,
    #[account(
        init,
        payer = registrar,
        space = 8 + OffereeEventRecord::INIT_SPACE,
        seeds = [b"offeree-event", offeree.key().as_ref(), &offeree.event_count.to_le_bytes()],
        bump
    )]
    pub event_record: Account<'info, OffereeEventRecord>,
}

#[derive(Accounts)]
pub struct CloseOffering<'info> {
    pub issuer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"offering", offering.asset.as_ref(), offering.offering_id.as_ref()],
        bump = offering.bump,
        has_one = issuer @ ErrorCode::Unauthorized
    )]
    pub offering: Account<'info, Offering>,
}

#[account]
#[derive(InitSpace)]
pub struct RegistrarAuthorization {
    pub asset: Pubkey,
    pub registrar: Pubkey,
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offering {
    pub asset: Pubkey,
    pub issuer: Pubkey,
    pub offering_id: [u8; 32],
    pub fiscal_period: [u8; 16],
    pub terms_commitment: [u8; 32],
    pub administrative_offeree_limit: Option<u32>,
    pub registered_count: u32,
    pub invited_count: u32,
    pub applied_count: u32,
    pub allotted_count: u32,
    pub closed: bool,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offeree {
    pub offering: Pubkey,
    pub offeree_ref: [u8; 32],
    pub state: OffereeState,
    pub invitation_commitment: Option<[u8; 32]>,
    pub application_commitment: Option<[u8; 32]>,
    pub allotment_commitment: Option<[u8; 32]>,
    pub allotment_amount: Option<u64>,
    pub event_count: u32,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct OffereeEventRecord {
    pub offeree: Pubkey,
    pub index: u32,
    pub event: OffereeEvent,
    pub commitment: [u8; 32],
    pub amount: Option<u64>,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum OffereeState { Registered, Invited, Applied, Allotted, Rejected, Withdrawn }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum OffereeEvent { Registered, Invite, Apply, Allot, Reject, Withdraw }

#[event]
pub struct RegistrarAuthorizationChanged { pub asset: Pubkey, pub registrar: Pubkey, pub active: bool }
#[event]
pub struct OfferingCreated { pub asset: Pubkey, pub offering: Pubkey, pub issuer: Pubkey }
#[event]
pub struct OffereeRegistered { pub offering: Pubkey, pub offeree_ref: [u8; 32] }
#[event]
pub struct OffereeEventRecorded { pub offering: Pubkey, pub offeree_ref: [u8; 32], pub event: OffereeEvent }
#[event]
pub struct OfferingClosedEvent { pub offering: Pubkey }
