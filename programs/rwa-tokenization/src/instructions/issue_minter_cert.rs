use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{spl_token_2022::instruction::AuthorityType, Token2022},
    token_interface::{
        mint_to, set_authority,
        spl_pod::optional_keys::OptionalNonZeroPubkey,
        spl_token_metadata_interface::state::{Field, TokenMetadata},
        token_metadata_initialize, token_metadata_update_field, Mint, MintTo, SetAuthority,
        TokenAccount, TokenMetadataInitialize, TokenMetadataUpdateField,
    },
};

use crate::{
    minter_controller::MinterController, update_account_minimum_lamports, GovernanceConfig,
    AVAILABLE_CREDITS_KEY, GOVERNANCE_CONFIG_SEED, MINTED_CREDITS_KEY, MINTER_NFT_SEED,
};

#[derive(Accounts)]
pub struct IssueMinterCert<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        has_one = authority,
        seeds = [GOVERNANCE_CONFIG_SEED],
        bump = config_account.bump,
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    #[account(
        init,
        payer = authority,
        space = 8 + MinterController::INIT_SPACE,
      seeds = [MINTER_NFT_SEED, mint.key.as_ref()],
      bump
    )]
    pub minter_controller: Box<Account<'info, MinterController>>,
    /// CHECK: This is nft keeper account
    pub receiver: AccountInfo<'info>,
    #[account(
      init,
      payer = authority,
      mint::token_program = token_program,
      mint::decimals = 0,
      mint::authority = minter_controller,
      extensions::metadata_pointer::authority = minter_controller,
      extensions::metadata_pointer::metadata_address = mint,
      extensions::close_authority::authority = minter_controller,
      extensions::permanent_delegate::delegate = minter_controller,
      seeds = [MINTER_NFT_SEED, receiver.key.as_ref()],
      bump
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
      init,
      payer = authority,
      associated_token::token_program = token_program,
      associated_token::mint = mint,
      associated_token::authority = receiver
    )]
    pub receiver_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> IssueMinterCert<'info> {
    pub fn handler(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        bumps: &IssueMinterCertBumps,
    ) -> Result<()> {
        self.minter_controller.set_inner(MinterController {
            mint: self.mint.key(),
            user: self.receiver.key(),
            bump: bumps.minter_controller,
        });
        self.update_account_lamports_by_metadata(name.clone(), symbol.clone(), uri.clone())?;
        self.init_nft_metadata(name, symbol, uri)?;
        self.mint_and_send_nft()?;
        Ok(())
    }

    fn mint_and_send_nft(&mut self) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            MINTER_NFT_SEED,
            mint_key.as_ref(),
            &[self.minter_controller.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // nint just 1 token, because it's a NFT
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.receiver_token_account.to_account_info(),
                    authority: self.minter_controller.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        // Freeze mint authority to prevent minting more tokens
        set_authority(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                SetAuthority {
                    current_authority: self.minter_controller.to_account_info(),
                    account_or_mint: self.mint.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }

    fn init_nft_metadata(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            MINTER_NFT_SEED,
            mint_key.as_ref(),
            &[self.minter_controller.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        // init token metadata

        token_metadata_initialize(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.minter_controller.to_account_info(),
                    update_authority: self.minter_controller.to_account_info(),
                    metadata: self.mint.to_account_info(),
                },
                signer_seeds,
            ),
            name,
            symbol,
            uri,
        )?;

        token_metadata_update_field(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    metadata: self.mint.to_account_info(),
                    update_authority: self.minter_controller.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key(AVAILABLE_CREDITS_KEY.to_string()),
            "0".to_string(),
        )?;

        token_metadata_update_field(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    metadata: self.mint.to_account_info(),
                    update_authority: self.minter_controller.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key(MINTED_CREDITS_KEY.to_string()),
            "0".to_string(),
        )?;

        Ok(())
    }

    fn update_account_lamports_by_metadata(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey(self.minter_controller.key()),
            mint: self.mint.key(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            additional_metadata: vec![
                (AVAILABLE_CREDITS_KEY.to_string(), "0".to_string()),
                (MINTED_CREDITS_KEY.to_string(), "0".to_string()),
            ],
        };

        let meta_data_space = token_metadata.tlv_size_of().unwrap();

        update_account_minimum_lamports(
            self.mint.to_account_info(),
            self.authority.to_account_info(),
            self.system_program.to_account_info(),
            meta_data_space,
        )?;

        Ok(())
    }
}
