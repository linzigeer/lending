use crate::constants::{MAXIMUM_AGE, SOL_USD_FEED_ID, TREASURY, USDC_USD_FEED_ID};
use crate::enums::MintKind;
use crate::errors::ErrorCode;
use crate::float_utils::calc_base_sum_interest;
use crate::states::{Bank, User};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump = bank_account.bump,
    )]
    pub bank_account: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [TREASURY.as_bytes(), mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = bank_token_account
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [signer.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, User>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub price_update: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn borrow_handler(
    ctx: Context<Borrow>,
    collateralized_kind: MintKind,
    to_borrow_kind: MintKind,
    to_borrow_value: f64,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let bank_account = &mut ctx.accounts.bank_account;
    let price_updator = &mut ctx.accounts.price_update;
    let clock = Clock::get()?;
    let sol_feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
    let sol_price = price_updator.get_price_no_older_than(&clock, MAXIMUM_AGE, &sol_feed_id)?;
    let usdc_feed_id = get_feed_id_from_hex(USDC_USD_FEED_ID)?;
    let usdc_price = price_updator.get_price_no_older_than(&clock, MAXIMUM_AGE, &usdc_feed_id)?;

    let borrowed_target_price = match to_borrow_kind {
        MintKind::SOL => sol_price,
        MintKind::USDC => usdc_price,
    };

    let collateralized_value = match collateralized_kind {
        MintKind::SOL => {
            let collateralized_amount = user_account.deposited_sol_amount;
            let collateralized_amount = calc_base_sum_interest(
                collateralized_amount,
                bank_account.deposited_interest_ratio,
                user_account.last_updated,
                4,
            );
            let collateralized_value = collateralized_amount * (sol_price.price as f64);
            collateralized_value
        }
        MintKind::USDC => {
            let collateralized_amount = user_account.deposited_usdc_amount;
            let collateralized_amount = calc_base_sum_interest(
                collateralized_amount,
                bank_account.deposited_interest_ratio,
                user_account.last_updated,
                4,
            );
            let collateralized_value = collateralized_amount * usdc_price.price as f64;
            collateralized_value
        }
    };
    if collateralized_value <= 0.0 {
        return Err(ErrorCode::DepositedValueLessOrEqualZero.into());
    }

    if to_borrow_value > collateralized_value * bank_account.max_ltv {
        return Err(ErrorCode::NotEnoughLiquidationAssets.into());
    }

    let program = ctx.accounts.token_program.to_account_info();
    let accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
    };
    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        TREASURY.as_bytes(),
        mint_key.as_ref(),
        &[ctx.bumps.bank_token_account],
    ]];
    let cpi_ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
    let decimals = ctx.accounts.mint.decimals;
    let amount = to_borrow_value / borrowed_target_price.price as f64;
    transfer_checked(cpi_ctx, amount as u64, decimals)?;

    Ok(())
}
