use anchor_lang::prelude::*;
pub mod constants;
pub mod enums;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

pub use constants::*;
pub use enums::*;
pub use errors::ErrorCode;
pub use instructions::*;
pub use states::*;
pub use utils::*;

declare_id!("6yT6ZAak1W9iD7mQSvoyHQzXhbdJqktqyFMbhkuAYA2a");

#[program]
pub mod lending {
    use super::*;

    pub fn process_init_bank(
        ctx: Context<InitBank>,
        liquidate_threshold: f64,
        liquidate_bonus: f64,
        liquidate_close_factor: f64,
        max_ltv: f64,
        deposited_interest_ratio: f64,
        borrowed_interest_ratio: f64,
    ) -> Result<()> {
        instructions::init_bank_handler(
            ctx,
            liquidate_threshold,
            liquidate_bonus,
            liquidate_close_factor,
            max_ltv,
            deposited_interest_ratio,
            borrowed_interest_ratio,
        )?;

        Ok(())
    }

    pub fn process_init_user(ctx: Context<InitUser>) -> Result<()> {
        instructions::init_user_handler(ctx)?;

        Ok(())
    }

    pub fn process_deposit(ctx: Context<Deposit>, mint: String, amount: u64) -> Result<()> {
        let mint_kind = match mint.as_str() {
            SOL => MintKind::SOL,
            USDC => MintKind::USDC,
            _ => {
                return Err(ErrorCode::UnsupportedMintKind.into());
            }
        };
        instructions::deposit_handler(ctx, mint_kind, amount)?;

        Ok(())
    }

    pub fn process_borrow(
        ctx: Context<Borrow>,
        collateralized_kind: String,
        to_borrow_kind: String,
        to_borrow_value: f64,
    ) -> Result<()> {
        let collateralized_kind = match collateralized_kind.as_str() {
            SOL => MintKind::SOL,
            USDC => MintKind::USDC,
            _ => {
                return Err(ErrorCode::UnsupportedMintKind.into());
            }
        };
        let to_borrow_kind = match to_borrow_kind.as_str() {
            SOL => MintKind::SOL,
            USDC => MintKind::USDC,
            _ => {
                return Err(ErrorCode::UnsupportedMintKind.into());
            }
        };
        if collateralized_kind == to_borrow_kind {
            return Err(ErrorCode::BorrowNotAllowed.into());
        }
        instructions::borrow_handler(ctx, collateralized_kind, to_borrow_kind, to_borrow_value)?;

        Ok(())
    }

    pub fn process_repay(
        ctx: Context<Repay>,
        repay_kind: String,
        to_repay_amount: u64,
    ) -> Result<()> {
        let repay_kind = match repay_kind.as_str() {
            SOL => MintKind::SOL,
            USDC => MintKind::USDC,
            _ => {
                return Err(ErrorCode::UnsupportedMintKind.into());
            }
        };
        instructions::repay_handler(ctx, repay_kind, to_repay_amount)?;

        Ok(())
    }
}
