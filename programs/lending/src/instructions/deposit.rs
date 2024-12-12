use crate::errors::ErrorCode;
use crate::enums::{ShareOp, MintKind};
use crate::float_utils::calc_new_total_shares;
use crate::states::{Bank, User};
use crate::constants::TREASURY;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        bump = user_account.bump
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

pub fn deposit_handler(
    ctx: Context<Deposit>,
    mint_kind: MintKind,
    current_deposit_amount: u64,
) -> Result<()> {
    require!(
        current_deposit_amount > 0,
        ErrorCode::DepositedAmountLessOrEqualZero
    );
    let user_account = &mut ctx.accounts.user_account;
    let bank_account = &mut ctx.accounts.bank_account;

    let total_deposited_amount = bank_account.total_deposited_amount;
    let total_deposited_shares = bank_account.total_deposited_shares;
    let user_deposited_shares;

    if total_deposited_amount == 0 {
        user_deposited_shares = current_deposit_amount as f64;
        bank_account.total_deposited_amount = current_deposit_amount;
        bank_account.total_deposited_shares = current_deposit_amount as f64;
    } else {
        user_deposited_shares = calc_new_total_shares(current_deposit_amount, total_deposited_amount, total_deposited_shares, 4, ShareOp::Increase);
        bank_account.total_deposited_amount += current_deposit_amount;
        bank_account.total_deposited_shares = user_deposited_shares;
    }
    match mint_kind {
        MintKind::SOL => {
            user_account.deposited_sol_amount += current_deposit_amount;
            user_account.deposited_sol_shares += user_deposited_shares;
        }
        MintKind::USDC => {
            user_account.deposited_usdc_amount += current_deposit_amount;
            user_account.deposited_usdc_shares += user_deposited_shares;
        }
    }

    let accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(program, accounts);
    let decimals = ctx.accounts.mint.decimals;
    transfer_checked(cpi_ctx, current_deposit_amount, decimals)?;

    Ok(())
}
