use crate::constants::TREASURY;
use crate::enums::MintKind;
use crate::errors::ErrorCode;
use crate::float_utils::{calc_base_sum_interest, calc_change_shares};
use crate::states::{Bank, User};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Repay<'info> {
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn repay_handler(
    ctx: Context<Repay>,
    repay_kind: MintKind,
    to_repay_amount: u64,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let bank_account = &mut ctx.accounts.bank_account;
    let user_borrowed_amount = match repay_kind {
        MintKind::SOL => {
            if user_account.borrowed_sol_amount == 0 {
                return Err(ErrorCode::NoNeedToRepay.into());
            }
            user_account.borrowed_sol_amount
        }
        MintKind::USDC => {
            if user_account.borrowed_usdc_amount == 0 {
                return Err(ErrorCode::NoNeedToRepay.into());
            }
            user_account.borrowed_usdc_amount
        }
    };

    let changed_shares = calc_change_shares(
        to_repay_amount,
        user_borrowed_amount,
        bank_account.total_borrowed_shares,
        4,
    );

    let user_borrowed_amount = calc_base_sum_interest(
        user_borrowed_amount,
        bank_account.borrowed_interest_ratio,
        user_account.last_updated,
        4,
    );

    if to_repay_amount as f64 > user_borrowed_amount {
        return Err(ErrorCode::RepayExceedBorrowed.into());
    }

    let program = ctx.accounts.token_program.to_account_info();
    let accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(program, accounts);
    let decimals = ctx.accounts.mint.decimals;
    transfer_checked(cpi_ctx, to_repay_amount, decimals)?;

    bank_account.total_borrowed_shares -= changed_shares;
    match repay_kind {
        MintKind::SOL => user_account.borrowed_sol_shares -= changed_shares,
        MintKind::USDC => user_account.borrowed_usdc_shares -= changed_shares,
    }

    Ok(())
}
