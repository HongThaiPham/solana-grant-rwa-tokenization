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
    error::MyErrorCode, MintAuthority, MinterController, AVAILABLE_CREDITS_KEY, MINTED_CREDITS_KEY,
    MINTER_NFT_SEED, MINT_AUTHORITY_SEED,
};

#[derive(Accounts)]
pub struct MintRwaToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub minter: Signer<'info>,
    /// CHECK: This is nft keeper account
    pub receiver: AccountInfo<'info>,
    #[account(
        constraint = minter_controller.mint == minter_nft_mint.key(),
        constraint = minter_controller.user == minter.key(),
      seeds = [MINTER_NFT_SEED, minter_nft_mint.key().as_ref()],
      bump = minter_controller.bump
    )]
    pub minter_controller: Box<Account<'info, MinterController>>,
    #[account(
        constraint = mint_authority.mint == rwa_mint.key(),
        // constraint = mint_authority.transfer_hook == transfer_hook_program.key(),
        seeds = [MINT_AUTHORITY_SEED, rwa_mint.key().as_ref()],
        bump = mint_authority.bump,
    )]
    pub mint_authority: Box<Account<'info, MintAuthority>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = mint_authority,
        mint::decimals = 0,
    )]
    pub rwa_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::token_program = token_program,
        associated_token::mint = rwa_mint,
        associated_token::authority = receiver
    )]
    pub receiver_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::decimals = 0,
        constraint = minter_nft_mint.supply == 1,
        seeds = [MINTER_NFT_SEED, rwa_mint.key().as_ref(), minter.key.as_ref()],
        bump
    )]
    pub minter_nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        associated_token::token_program = token_program,
        associated_token::mint = minter_nft_mint,
        associated_token::authority = minter
    )]
    pub minter_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    // /// CHECK: This is transfer hook program
    // pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintRwaToken<'info> {
    pub fn handler(&mut self, _symbol: String, amount: u64) -> Result<()> {
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

            let minter_nft_mint_key = self.minter_nft_mint.key();
            let seeds = &[
                MINTER_NFT_SEED,
                minter_nft_mint_key.as_ref(),
                &[self.minter_controller.bump],
            ];
            let signer_seeds = &[&seeds[..]];

            token_metadata_update_field(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenMetadataUpdateField {
                        metadata: self.minter_nft_mint.to_account_info(),
                        program_id: self.token_program.to_account_info(),
                        update_authority: self.minter_controller.to_account_info(),
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
                        update_authority: self.minter_controller.to_account_info(),
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
        let mint_key = self.rwa_mint.key();
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
                    mint: self.rwa_mint.to_account_info(),
                    to: self.receiver_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        Ok(())
    }
}
