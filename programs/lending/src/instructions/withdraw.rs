use crate::constants::TREASURY;
use crate::enums::MintKind;
use crate::errors::ErrorCode;
use crate::float_utils::{calc_base_sum_interest, calc_change_shares};
use crate::states::{Bank, User};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct WithDraw<'info> {
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
        token::authority = bank_token_account,
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

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn withdraw_handler(ctx: Context<WithDraw>, mint_kind: MintKind, amount: u64) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let bank_account = &mut ctx.accounts.bank_account;
    let bank_total_deposited = bank_account.total_deposited_amount;
    let bank_total_shares = bank_account.total_deposited_shares;

    let user_deposited_amount_without_interest = match mint_kind {
        MintKind::SOL => {
            if user_account.deposited_sol_amount <= 0 {
                return Err(ErrorCode::NotEnoughBalance.into());
            }
            user_account.deposited_sol_amount
        }
        MintKind::USDC => {
            if user_account.deposited_usdc_amount <= 0 {
                return Err(ErrorCode::NotEnoughBalance.into());
            }
            user_account.deposited_usdc_amount
        }
    };
    let user_deposited_amount_with_interest = calc_base_sum_interest(
        user_deposited_amount_without_interest,
        bank_account.deposited_interest_ratio,
        user_account.last_updated,
        4,
    );
    if amount as f64 > user_deposited_amount_with_interest {
        return Err(ErrorCode::NotEnoughBalance.into()); 
    }

    let change_shares = calc_change_shares(amount, bank_total_deposited, bank_total_shares, 4);

    Ok(())
}
