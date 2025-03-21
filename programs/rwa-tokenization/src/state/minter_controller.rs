use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MinterController {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub bump: u8,
}
