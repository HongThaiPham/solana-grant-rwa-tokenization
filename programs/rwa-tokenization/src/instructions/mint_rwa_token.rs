use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        spl_token_2022::{
            self,
            extension::{BaseStateWithExtensions, StateWithExtensions},
        },
        Token2022,
    },
    token_interface::{
        mint_to,
        spl_token_metadata_interface::state::{Field, TokenMetadata},
        token_metadata_update_field, Mint, MintTo, TokenAccount, TokenMetadataUpdateField,
    },
};

use crate::{
    error::MyErrorCode, GovernanceConfig, MintAuthority, AVAILABLE_CREDITS_KEY,
    CARBON_CREDIT_TOKEN_SEED, GOVERNANCE_CONFIG_SEED, MINTED_CREDITS_KEY, MINTER_NFT_SEED,
    MINT_AUTHORITY_SEED,
};

#[derive(Accounts)]
pub struct MintRwaToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        seeds = [GOVERNANCE_CONFIG_SEED],
        bump = config_account.bump,
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    #[account(
        constraint = mint_authority.authority == creator.key(),
        constraint = mint_authority.mint == mint.key(),
        // constraint = mint_authority.transfer_hook == transfer_hook_program.key(),
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump = mint_authority.bump,
    )]
    pub mint_authority: Box<Account<'info, MintAuthority>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = mint_authority,
        mint::decimals = 0,
        extensions::metadata_pointer::authority = mint_authority,
        extensions::metadata_pointer::metadata_address = mint,
        extensions::close_authority::authority = mint_authority,
        // extensions::transfer_hook::program_id = transfer_hook_program,
        // extensions::transfer_hook::authority = mint_authority,
        seeds = [CARBON_CREDIT_TOKEN_SEED, minter_nft_mint.key().as_ref()],
        bump
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::decimals = 0,
        constraint = minter_nft_mint.supply == 1,
        seeds = [MINTER_NFT_SEED, creator.key.as_ref()],
        bump
    )]
    pub minter_nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        associated_token::token_program = token_program,
        associated_token::mint = minter_nft_mint,
        associated_token::authority = creator
    )]
    pub minter_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    // /// CHECK: This is transfer hook program
    // pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintRwaToken<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, MyErrorCode::InvalidAmount);
        // read metadata from nft mint

        let additional_metadata = self.get_additional_metadata()?;
        let available_credits = additional_metadata
            .iter()
            .find(|(key, _)| key == AVAILABLE_CREDITS_KEY);

        if let Some((_, value)) = available_credits {
            let available_credits = value.parse::<u64>().map_err(|_| MyErrorCode::NoCredits)?;
            require!(
                available_credits >= amount,
                MyErrorCode::InsufficientCredits
            );

            self.mint_to_recevier(amount)?;

            let seeds = &[GOVERNANCE_CONFIG_SEED, &[self.config_account.bump]];
            let signer_seeds = &[&seeds[..]];

            token_metadata_update_field(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenMetadataUpdateField {
                        metadata: self.minter_nft_mint.to_account_info(),
                        program_id: self.token_program.to_account_info(),
                        update_authority: self.config_account.to_account_info(),
                    },
                    signer_seeds,
                ),
                Field::Key(AVAILABLE_CREDITS_KEY.to_string()),
                available_credits.checked_sub(amount).unwrap().to_string(),
            )?;
            available_credits
                .checked_sub(amount)
                .ok_or(MyErrorCode::Overflow)?
                .to_string();

            let minted_credits = additional_metadata
                .iter()
                .find(|(key, _)| key == MINTED_CREDITS_KEY);

            let mut updated_minted_credits = amount;
            if let Some((_, value)) = minted_credits {
                let minted_credits = value.parse::<u64>().unwrap();
                updated_minted_credits = minted_credits
                    .checked_add(amount)
                    .ok_or(MyErrorCode::Overflow)?;
            }

            token_metadata_update_field(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenMetadataUpdateField {
                        metadata: self.minter_nft_mint.to_account_info(),
                        program_id: self.token_program.to_account_info(),
                        update_authority: self.config_account.to_account_info(),
                    },
                    signer_seeds,
                ),
                Field::Key(MINTED_CREDITS_KEY.to_string()),
                updated_minted_credits.to_string(),
            )?;
        } else {
            return Err(MyErrorCode::InsufficientCredits.into());
        }
        Ok(())
    }

    fn get_additional_metadata(&self) -> Result<Vec<(String, String)>> {
        let nft_minter = self.minter_nft_mint.to_account_info();
        let nft_minter_data = nft_minter.data.borrow();
        let mint_with_extension =
            StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&nft_minter_data)?;
        let metadata = mint_with_extension.get_variable_len_extension::<TokenMetadata>()?;
        Ok(metadata.additional_metadata)
    }

    fn mint_to_recevier(&mut self, amount: u64) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            MINT_AUTHORITY_SEED,
            mint_key.as_ref(),
            &[self.mint_authority.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.creator_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        Ok(())
    }
}
