use crate::constants::{DISCRIMINATOR, TREASURY};
use crate::errors::ErrorCode;
use crate::states::{Bank, BankBuilder};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct InitBank<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        seeds = [mint.key().as_ref()],
        space = DISCRIMINATOR + Bank::INIT_SPACE,
        bump,
    )]
    pub bank_account: Account<'info, Bank>,

    #[account(
        init,
        payer = signer,
        seeds = [TREASURY.as_bytes(), mint.key().as_ref()],
        token::mint = mint,
        token::authority = bank_token_account,
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn init_bank_handler(
    ctx: Context<InitBank>,
    liquidate_threshold: f64,
    liquidate_bonus: f64,
    liquidate_close_factor: f64,
    max_ltv: f64,
    deposited_interest_ratio: f64,
    borrowed_interest_ratio: f64,
) -> Result<()> {
    msg!("init_bank_handler");
    let init_bank = BankBuilder::default()
        .authority(ctx.accounts.signer.key())
        .mint(ctx.accounts.mint.key())
        .liquidate_threshold(liquidate_threshold)
        .liquidate_bonus(liquidate_bonus)
        .liquidate_close_factor(liquidate_close_factor)
        .max_ltv(max_ltv)
        .deposited_interest_ratio(deposited_interest_ratio)
        .borrowed_interest_ratio(borrowed_interest_ratio)
        .last_updated(Clock::get()?.unix_timestamp)
        .build()
        .map_err(|_| ErrorCode::BuilderError)?;

    *ctx.accounts.bank_account = init_bank;

    Ok(())
}
