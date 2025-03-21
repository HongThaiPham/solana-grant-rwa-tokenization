use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MintAuthority {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub transfer_hook: Option<Pubkey>,
    pub bump: u8,
}
