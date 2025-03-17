use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, 
    token_2022::{spl_token_2022::{self, extension::ExtensionType}, Token2022}, 
    token_interface::{spl_pod::optional_keys::OptionalNonZeroPubkey, spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint, TokenAccount, TokenMetadataInitialize}};

use crate::{MintAuthority, MINTER_NFT_SEED, MINT_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct InitCarbonToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + MintAuthority::INIT_SPACE,
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump
    )]
    pub mint_authority: Box<Account<'info, MintAuthority>>,
    #[account(
        init,
        payer = payer,
        mint::token_program = token_program,
        mint::authority = mint_authority,
        mint::decimals = 0, 
        extensions::metadata_pointer::authority = mint_authority,
        extensions::metadata_pointer::metadata_address = mint,
        extensions::close_authority::authority = mint_authority, 
        extensions::transfer_hook::program_id = transfer_hook_program,
        extensions::transfer_hook::authority = mint_authority  
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
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
    /// CHECK: This is transfer hook program
    pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> InitCarbonToken<'info> {
    pub fn handler(&mut self, name: String, symbol: String, uri: String, bump: &InitCarbonTokenBumps) -> Result<()> {
        self.mint_authority.set_inner(
            MintAuthority { 
                authority: self.creator.key(),
                 mint: self.mint.key(), 
                 transfer_hook: self.transfer_hook_program.key(), 
                 bump: bump.mint_authority });
        self.update_account_lamports_by_extensions(name.clone(), symbol.clone(), uri.clone())?;
        self.init_nft_metadata(name, symbol, uri)?;
        Ok(())
    }

    fn init_nft_metadata(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds = &[MINT_AUTHORITY_SEED, mint_key.as_ref(), &[self.mint_authority.bump]];
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

    fn update_account_lamports_by_extensions(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey(self.mint_authority.key()),
            mint: self.mint.key(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            ..Default::default()
        };

        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
            ExtensionType::TransferHook,
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
                    from: self.payer.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports_required,
        )?;
        Ok(())
    }
    
}