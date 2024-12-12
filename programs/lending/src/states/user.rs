use anchor_lang::prelude::*;
use derive_builder::Builder;

#[account]
#[derive(InitSpace, Default, Debug, Builder)]
pub struct User {
    pub owner: Pubkey,
    pub deposited_usdc_amount: u64,
    pub deposited_usdc_shares: f64,
    pub borrowed_usdc_amount: u64,
    pub borrowed_usdc_shares: f64,
    pub deposited_sol_amount: u64,
    pub deposited_sol_shares: f64,
    pub borrowed_sol_amount: u64,
    pub borrowed_sol_shares: f64,
    pub deposited_mint: Pubkey,
    pub borrowed_mint: Pubkey,
    pub health_factor: f64,
    pub bump: u8,
    pub last_updated: i64,
}
