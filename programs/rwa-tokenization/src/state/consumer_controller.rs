use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ConsumerController {
    pub rwa_mint: Pubkey,
    pub mint: Pubkey,
    pub user: Pubkey,
    pub bump: u8,
}
