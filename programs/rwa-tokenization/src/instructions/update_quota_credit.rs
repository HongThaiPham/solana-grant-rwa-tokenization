use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        spl_token_metadata_interface::state::Field, token_metadata_update_field, Mint,
        TokenMetadataUpdateField,
    },
};

use crate::{GovernanceConfig, AVAILABLE_CREDITS_KEY, GOVERNANCE_CONFIG_SEED, MINTER_NFT_SEED};

#[derive(Accounts)]
pub struct UpdateQuotaCredit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        has_one = authority,
        seeds = [GOVERNANCE_CONFIG_SEED],
        bump = config_account.bump,
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    /// CHECK: This is nft keeper account
    pub receiver: AccountInfo<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::decimals = 0,
        seeds = [MINTER_NFT_SEED, receiver.key.as_ref()],
        bump
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> UpdateQuotaCredit<'info> {
    pub fn handler(&mut self, new_credit: u64) -> Result<()> {
        let seeds = &[GOVERNANCE_CONFIG_SEED, &[self.config_account.bump]];
        let signer_seeds = &[&seeds[..]];
        token_metadata_update_field(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    program_id: self.token_program.to_account_info(),
                    metadata: self.mint.to_account_info(),
                    update_authority: self.config_account.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key(AVAILABLE_CREDITS_KEY.to_string()),
            new_credit.to_string(),
        )?;
        Ok(())
    }
}
