use anchor_lang::prelude::*;
#[error_code]
pub enum ErrorCode {
    #[msg("build error occurred!")]
    BuilderError,

    #[msg("Get clock time error!")]
    GetClockTimeError,

    #[msg("Deposited Amount less or equal than zero!")]
    DepositedAmountLessOrEqualZero,

    #[msg("Deposited Value less or equal than zero!")]
    DepositedValueLessOrEqualZero,

    #[msg("Unsupported mint kind!")]
    UnsupportedMintKind,

    #[msg("Not enough deposited assets!")]
    NotEnoughLiquidationAssets,

    #[msg("No need to repay!")]
    NoNeedToRepay,

    #[msg("repay amount exceeds borrowed amount")]
    RepayExceedBorrowed,

    #[msg("Borrow not allowed!")]
    BorrowNotAllowed,

    #[msg("Not enough balance!")]
    NotEnoughBalance,
}
