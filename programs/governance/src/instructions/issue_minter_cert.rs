use std::vec;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        spl_token_2022::{self, extension::ExtensionType, instruction::AuthorityType},
        Token2022,
    },
    token_interface::{
        mint_to, set_authority,
        spl_pod::optional_keys::OptionalNonZeroPubkey,
        spl_token_metadata_interface::state::{Field, TokenMetadata},
        token_metadata_initialize, token_metadata_update_field, Mint, MintTo, SetAuthority,
        TokenAccount, TokenMetadataInitialize, TokenMetadataUpdateField,
    },
};

use crate::{GovernanceConfig, GOVERNANCE_CONFIG_SEED, MINTER_NFT_SEED};

#[derive(Accounts)]
pub struct IssueMinterCert<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
      has_one = authority,
      seeds = [GOVERNANCE_CONFIG_SEED],
      bump
    )]
    pub config_account: Box<Account<'info, GovernanceConfig>>,
    /// CHECK: This is nft keeper account
    pub receiver: AccountInfo<'info>,
    #[account(
      init,
      payer = authority,
      mint::token_program = token_program,
      mint::decimals = 0,
      mint::authority = config_account,
      extensions::metadata_pointer::authority = config_account,
      extensions::metadata_pointer::metadata_address = mint,
      extensions::close_authority::authority = config_account,
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
    pub fn handler(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        self.update_account_lamports_by_extensions(name.clone(), symbol.clone(), uri.clone())?;
        self.init_nft_collection_metadata(name, symbol, uri)?;
        self.mint_and_send_nft()?;
        Ok(())
    }

    fn mint_and_send_nft(&mut self) -> Result<()> {
        let seeds = &[GOVERNANCE_CONFIG_SEED, &[self.config_account.bump]];
        let signer_seeds = &[&seeds[..]];

        // nint just 1 token, because it's a NFT
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.receiver_token_account.to_account_info(),
                    authority: self.config_account.to_account_info(),
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
                    current_authority: self.config_account.to_account_info(),
                    account_or_mint: self.mint.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }

    fn init_nft_collection_metadata(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let seeds = &[GOVERNANCE_CONFIG_SEED, &[self.config_account.bump]];
        let signer_seeds = &[&seeds[..]];
        // init token metadata

        token_metadata_initialize(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.config_account.to_account_info(),
                    update_authority: self.config_account.to_account_info(),
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
                    update_authority: self.config_account.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key("available_credits".to_string()),
            "0".to_string(),
        )?;

        token_metadata_update_field(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    metadata: self.mint.to_account_info(),
                    update_authority: self.config_account.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key("minted_credits".to_string()),
            "0".to_string(),
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
            update_authority: OptionalNonZeroPubkey(self.config_account.key()),
            mint: self.mint.key(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            additional_metadata: vec![
                ("available_credits".to_string(), "0".to_string()),
                ("minted_credits".to_string(), "0".to_string()),
            ],
        };

        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
        ])
        .unwrap();

        let meta_data_space = token_metadata.tlv_size_of().unwrap();

        let total_space = space + meta_data_space;

        let lamports_required = (Rent::get()?).minimum_balance(total_space);

        msg!(
            "Create Mint and metadata account size with cost: {} lamports: {}",
            total_space as u64,
            lamports_required
        );

        system_program::transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                system_program::Transfer {
                    from: self.authority.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports_required,
        )?;
        Ok(())
    }
}
