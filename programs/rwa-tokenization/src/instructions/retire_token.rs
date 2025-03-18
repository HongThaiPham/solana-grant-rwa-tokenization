use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        burn_checked,
        spl_token_2022::{self, extension::ExtensionType, instruction::AuthorityType},
        BurnChecked, Token2022,
    },
    token_interface::{
        mint_to, set_authority,
        spl_pod::optional_keys::OptionalNonZeroPubkey,
        spl_token_metadata_interface::state::{Field, TokenMetadata},
        token_metadata_initialize, token_metadata_update_field, Mint, MintTo, SetAuthority,
        TokenAccount, TokenMetadataInitialize, TokenMetadataUpdateField,
    },
};

use crate::{
    error::MyErrorCode, update_account_minimun_lamports, MintAuthority, MINT_AUTHORITY_SEED,
    RETIRED_CREDITS_CERT_NAME, RETIRED_CREDITS_CERT_SYMBOL, RETIRED_CREDITS_KEY,
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct RetireToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub consumer: Signer<'info>,
    #[account(
      constraint = mint_authority.mint == mint.key(),
      constraint = mint_authority.transfer_hook == transfer_hook_program.key(),
      seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
      bump = mint_authority.bump,
    )]
    pub mint_authority: Box<Account<'info, MintAuthority>>,
    #[account(
      mut,
      mint::token_program = token_program,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = consumer
    )]
    pub consumer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      init,
      payer = payer,
      mint::token_program = token_program,
      mint::decimals = 0,
      mint::authority = mint_authority,
      extensions::metadata_pointer::authority = mint_authority,
      extensions::metadata_pointer::metadata_address = mint,
      extensions::close_authority::authority = mint_authority,
      extensions::permanent_delegate::delegate = mint_authority,
    )]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
      init,
      payer = payer,
      associated_token::token_program = token_program,
      associated_token::mint = nft_mint,
      associated_token::authority = consumer
    )]
    pub consumer_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: This is transfer hook program
    pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> RetireToken<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, MyErrorCode::InvalidAmount);
        let consumer_token_account = &self.consumer_token_account;
        require!(
            consumer_token_account.amount >= amount,
            MyErrorCode::InsufficientBalance
        );
        self.burn_token(amount)?;
        let name = RETIRED_CREDITS_CERT_NAME.to_string();
        let symbol = RETIRED_CREDITS_CERT_SYMBOL.to_string();
        let uri = "".to_string();

        self.update_account_lamports_by_extensions(
            name.clone(),
            symbol.clone(),
            uri.clone(),
            amount,
            &[ExtensionType::MetadataPointer],
        )?;
        self.init_nft_metadata(name, symbol, uri, amount)?;
        self.mint_and_send_nft()?;
        Ok(())
    }

    fn mint_and_send_nft(&mut self) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[
            MINT_AUTHORITY_SEED,
            mint_key.as_ref(),
            &[self.mint_authority.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // nint just 1 token, because it's a NFT
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.nft_mint.to_account_info(),
                    to: self.consumer_nft_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
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
                    current_authority: self.mint_authority.to_account_info(),
                    account_or_mint: self.nft_mint.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }

    fn init_nft_metadata(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        amount: u64,
    ) -> Result<()> {
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
                    mint: self.nft_mint.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.mint_authority.to_account_info(),
                    update_authority: self.mint_authority.to_account_info(),
                    metadata: self.nft_mint.to_account_info(),
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
                    metadata: self.nft_mint.to_account_info(),
                    update_authority: self.mint_authority.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            Field::Key(RETIRED_CREDITS_KEY.to_string()),
            amount.to_string(),
        )?;

        Ok(())
    }

    fn update_account_lamports_by_extensions(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        amount: u64,
        extension: &[ExtensionType],
    ) -> Result<()> {
        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey(self.mint_authority.key()),
            mint: self.nft_mint.key(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            additional_metadata: vec![(RETIRED_CREDITS_KEY.to_string(), amount.to_string())],
        };

        let space =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(extension)
                .unwrap();

        let meta_data_space = token_metadata.tlv_size_of().unwrap();

        let total_space = space + meta_data_space;

        update_account_minimun_lamports(
            self.nft_mint.to_account_info(),
            self.payer.to_account_info(),
            self.system_program.to_account_info(),
            total_space,
        )?;

        Ok(())
    }

    fn burn_token(&self, amount: u64) -> Result<()> {
        burn_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                BurnChecked {
                    from: self.consumer_token_account.to_account_info(),
                    authority: self.consumer.to_account_info(),
                    mint: self.mint.to_account_info(),
                },
            ),
            amount,
            self.mint.decimals,
        )?;
        Ok(())
    }
}
