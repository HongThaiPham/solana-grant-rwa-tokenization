use anchor_lang::prelude::*;

#[error_code]
pub enum MyErrorCode {
    #[msg("Insufficient credits")]
    InsufficientCredits,
}
