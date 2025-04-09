use anchor_lang::prelude::*;

#[error_code]
pub enum MyErrorCode {
    #[msg("Insufficient credits")]
    InsufficientCredits,
    #[msg("Overflow")]
    Overflow,
    #[msg("No credits")]
    NoCredits,
    InvalidAmount,
    InsufficientBalance,
    InvalidCredit,
}
