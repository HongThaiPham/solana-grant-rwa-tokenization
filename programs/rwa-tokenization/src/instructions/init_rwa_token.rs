use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{initialize_mint2, InitializeMint2, Token2022},
    token_interface::{
        metadata_pointer_initialize, mint_close_authority_initialize, token_metadata_initialize,
        transfer_fee_initialize, transfer_hook_initialize, MetadataPointerInitialize,
        MintCloseAuthorityInitialize, TokenMetadataInitialize, TransferFeeInitialize,
        TransferHookInitialize,
    },
};

use crate::{
    get_mint_space_with_extensions, update_account_lamports_to_minimum_balance, GovernanceConfig,
    MintAuthority, CARBON_CREDIT_TOKEN_SEED, GOVERNANCE_CONFIG_SEED, MINT_AUTHORITY_SEED,
};

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    decimals: u8,
    uri: String,
    is_close: bool,
    has_fee: bool,
)]
pub struct InitRwaToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        has_one = authority,
        seeds = [GOVERNANCE_CONFIG_SEED],
        bump
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    #[account(
        init,
        payer = authority,
        space = 8 + MintAuthority::INIT_SPACE,
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump
    )]
    pub mint_authority: Box<Account<'info, MintAuthority>>,
    /// CHECK: This is the mint account for the token
    #[account(
        init,
        payer = authority,
        space = get_mint_space_with_extensions(is_close, has_fee)?,
        seeds = [CARBON_CREDIT_TOKEN_SEED, symbol.as_ref()],
        bump,
        owner = token_program.key()
    )]
    pub mint: UncheckedAccount<'info>,
    /// CHECK: This is transfer hook program
    #[account(executable)]
    pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitRwaToken<'info> {
    pub fn handler(
        &mut self,
        name: String,
        symbol: String,
        decimals: u8,
        uri: String,
        is_close: bool,
        has_fee: bool,
        transfer_fee_basis_points: Option<u16>,
        maximum_fee: Option<u64>,
        bump: &InitRwaTokenBumps,
    ) -> Result<()> {
        self.mint_authority.set_inner(MintAuthority {
            authority: self.authority.key(),
            mint: self.mint.key(),
            transfer_hook: if is_close {
                Some(self.transfer_hook_program.key())
            } else {
                None
            },
            bump: bump.mint_authority,
        });
        self.init_extensions_and_mint(is_close, has_fee, transfer_fee_basis_points, maximum_fee)?;

        initialize_mint2(
            CpiContext::new(
                self.token_program.to_account_info(),
                InitializeMint2 {
                    mint: self.mint.to_account_info(),
                },
            ),
            decimals,                         // decimals
            &self.mint_authority.key(),       // mint authority
            Some(&self.mint_authority.key()), // freeze authority
        )?;

        self.init_nft_metadata(name, symbol, uri)?;

        update_account_lamports_to_minimum_balance(
            self.mint.to_account_info(),
            self.authority.to_account_info(),
            self.system_program.to_account_info(),
        )?;
        Ok(())
    }

    fn init_extensions_and_mint(
        &mut self,
        is_close: bool,
        has_fee: bool,
        transfer_fee_basis_points: Option<u16>,
        maximum_fee: Option<u64>,
    ) -> Result<()> {
        // Some extensions init must come before the instruction to initialize the mint data

        // Init metadata pointer
        metadata_pointer_initialize(
            CpiContext::new(
                self.token_program.to_account_info(),
                MetadataPointerInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            Some(self.mint_authority.key()),
            Some(self.mint.key()),
        )?;

        // init mint close authority

        mint_close_authority_initialize(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintCloseAuthorityInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            Some(&self.mint_authority.key()),
        )?;

        if is_close {
            // init transfer hook
            transfer_hook_initialize(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferHookInitialize {
                        mint: self.mint.to_account_info(),
                        token_program_id: self.token_program.to_account_info(),
                    },
                ),
                Some(self.mint_authority.key()),
                Some(self.transfer_hook_program.key()),
            )?;
        }

        if has_fee {
            // init transfer fee config
            transfer_fee_initialize(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferFeeInitialize {
                        token_program_id: self.token_program.to_account_info(),
                        mint: self.mint.to_account_info(),
                    },
                ),
                Some(&self.mint_authority.key()), // transfer fee config authority (update fee)
                Some(&self.mint_authority.key()), // withdraw authority (withdraw fees)
                transfer_fee_basis_points.unwrap(), // transfer fee basis points (% fee per transfer)
                maximum_fee.unwrap(), // maximum fee (maximum units of token per transfer)
            )?;
        }

        Ok(())
    }

    fn init_nft_metadata(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            MINT_AUTHORITY_SEED,
            mint_key.as_ref(),
            &[self.mint_authority.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        // init token metadata

        token_metadata_initialize(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.mint_authority.to_account_info(),
                    update_authority: self.mint_authority.to_account_info(),
                    metadata: self.mint.to_account_info(),
                },
                signer_seeds,
            ),
            name,
            symbol,
            uri,
        )?;

        Ok(())
    }
}
