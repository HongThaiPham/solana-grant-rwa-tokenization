use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        spl_token_2022::{self, extension::ExtensionType, instruction::AuthorityType},
        Token2022,
    },
    token_interface::{
        mint_to, set_authority, spl_pod::optional_keys::OptionalNonZeroPubkey,
        spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        MintTo, SetAuthority, TokenAccount, TokenMetadataInitialize,
    },
};

use crate::{update_account_minimum_lamports, ConsumerController, CONSUMER_NFT_SEED};

#[derive(Accounts)]
pub struct IssueConsumerCert<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + ConsumerController::INIT_SPACE,
      seeds = [CONSUMER_NFT_SEED, mint.key.as_ref()],
      bump
    )]
    pub consumer_controller: Box<Account<'info, ConsumerController>>,
    /// CHECK: This is nft keeper account
    pub receiver: AccountInfo<'info>,
    #[account(
      init,
      payer = authority,
      mint::token_program = token_program,
      mint::decimals = 0,
      mint::authority = consumer_controller,
      extensions::metadata_pointer::authority = consumer_controller,
      extensions::metadata_pointer::metadata_address = mint,
      extensions::close_authority::authority = consumer_controller,
      extensions::permanent_delegate::delegate = consumer_controller,
      seeds = [CONSUMER_NFT_SEED, receiver.key.as_ref()],
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

impl<'info> IssueConsumerCert<'info> {
    pub fn handler(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        bumps: &IssueConsumerCertBumps,
    ) -> Result<()> {
        self.consumer_controller.set_inner(ConsumerController {
            mint: self.mint.key(),
            user: self.receiver.key(),
            bump: bumps.consumer_controller,
        });
        self.update_account_lamports_by_extensions(name.clone(), symbol.clone(), uri.clone())?;
        self.init_nft_metadata(name, symbol, uri)?;
        self.mint_and_send_nft()?;
        Ok(())
    }

    fn mint_and_send_nft(&mut self) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            CONSUMER_NFT_SEED,
            mint_key.as_ref(),
            &[self.consumer_controller.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // nint just 1 token, because it's a NFT
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.receiver_token_account.to_account_info(),
                    authority: self.consumer_controller.to_account_info(),
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
                    current_authority: self.consumer_controller.to_account_info(),
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
            CONSUMER_NFT_SEED,
            mint_key.as_ref(),
            &[self.consumer_controller.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        // init token metadata

        token_metadata_initialize(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.consumer_controller.to_account_info(),
                    update_authority: self.consumer_controller.to_account_info(),
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

    fn update_account_lamports_by_extensions(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey(self.consumer_controller.key()),
            mint: self.mint.key(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            ..Default::default()
        };

        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
        ])
        .unwrap();

        let meta_data_space = token_metadata.tlv_size_of().unwrap();

        let total_space = space + meta_data_space;

        update_account_minimum_lamports(
            self.mint.to_account_info(),
            self.authority.to_account_info(),
            self.system_program.to_account_info(),
            total_space,
        )?;
        Ok(())
    }
}
