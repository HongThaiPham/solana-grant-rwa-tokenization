use anchor_lang::prelude::*;

use crate::{GovernanceConfig, GOVERNANCE_CONFIG_SEED};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub singer: Signer<'info>,
    #[account(
        init,
        payer = singer,
        space = 8 + GovernanceConfig::INIT_SPACE,
        seeds = [GOVERNANCE_CONFIG_SEED],
        bump
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeConfig<'info> {
    pub fn handler(&mut self, bumps: &InitializeConfigBumps) -> Result<()> {
        self.config_account.set_inner(GovernanceConfig {
            authority: self.singer.key(),
            is_initialized: true,
            bump: bumps.config_account,
        });
        Ok(())
    }
}
