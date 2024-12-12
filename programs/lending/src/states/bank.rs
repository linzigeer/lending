use anchor_lang::prelude::*;
use derive_builder::Builder;

#[account]
#[derive(InitSpace, Default, Builder, Debug)]
pub struct Bank {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub total_deposited_amount: u64,
    pub total_deposited_shares: f64,
    pub total_borrowed_amount: u64,
    pub total_borrowed_shares: f64,
    pub liquidate_threshold: f64,
    pub liquidate_bonus: f64,
    pub liquidate_close_factor: f64,
    pub max_ltv: f64,
    pub deposited_interest_ratio: f64,
    pub borrowed_interest_ratio: f64,
    pub bump: u8,
    pub last_updated: i64,
}
