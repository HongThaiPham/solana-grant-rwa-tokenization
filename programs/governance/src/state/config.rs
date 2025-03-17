use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GovernanceConfig {
    pub authority: Pubkey,
    pub bump: u8,
    pub is_initialized: bool,
}
