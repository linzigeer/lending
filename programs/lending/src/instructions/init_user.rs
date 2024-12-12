use crate::constants::DISCRIMINATOR;
use crate::errors::ErrorCode;
use crate::states::{User, UserBuilder};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [signer.key().as_ref()],
        space = DISCRIMINATOR + User::INIT_SPACE,
        bump,
    )]
    pub user_account: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn init_user_handler(ctx: Context<InitUser>) -> Result<()> {
    let init_user = UserBuilder::default()
        .owner(ctx.accounts.signer.key())
        .last_updated(Clock::get()?.unix_timestamp)
        .build()
        .map_err(|_| ErrorCode::BuilderError)?;

    *ctx.accounts.user_account = init_user;

    Ok(())
}
