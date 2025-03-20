use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        initialize_mint2, spl_token_2022::extension::ExtensionType, InitializeMint2, Token2022,
    },
    token_interface::{
        metadata_pointer_initialize, mint_close_authority_initialize, token_metadata_initialize,
        transfer_fee_initialize, transfer_hook_initialize, MetadataPointerInitialize, Mint,
        MintCloseAuthorityInitialize, TokenAccount, TokenMetadataInitialize, TransferFeeInitialize,
        TransferHookInitialize,
    },
};

use crate::{
    get_mint_space_with_extensions, update_account_lamports_to_minimum_balance, MintAuthority,
    CARBON_CREDIT_TOKEN_SEED, MINTER_NFT_SEED, MINT_AUTHORITY_SEED,
};

const EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::MetadataPointer,
    ExtensionType::MintCloseAuthority,
    ExtensionType::TransferHook,
    ExtensionType::TransferFeeConfig,
];

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
    /// CHECK: This is transfer hook program
    #[account(
        init,
        payer = payer,
        space = get_mint_space_with_extensions(EXTENSIONS)?,
        seeds = [CARBON_CREDIT_TOKEN_SEED, minter_nft_mint.key().as_ref()],
        bump,
        owner = token_program.key()
    )]
    pub mint: UncheckedAccount<'info>,
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
    #[account(executable)]
    pub transfer_hook_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitCarbonToken<'info> {
    pub fn handler(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
        bump: &InitCarbonTokenBumps,
    ) -> Result<()> {
        self.mint_authority.set_inner(MintAuthority {
            authority: self.creator.key(),
            mint: self.mint.key(),
            transfer_hook: self.transfer_hook_program.key(),
            bump: bump.mint_authority,
        });
        self.init_extensions_and_mint(transfer_fee_basis_points, maximum_fee)?;

        self.init_nft_metadata(name, symbol, uri)?;

        update_account_lamports_to_minimum_balance(
            self.mint.to_account_info(),
            self.payer.to_account_info(),
            self.system_program.to_account_info(),
        )?;
        Ok(())
    }

    fn init_extensions_and_mint(
        &mut self,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
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
            transfer_fee_basis_points,        // transfer fee basis points (% fee per transfer)
            maximum_fee,                      // maximum fee (maximum units of token per transfer)
        )?;

        initialize_mint2(
            CpiContext::new(
                self.token_program.to_account_info(),
                InitializeMint2 {
                    mint: self.mint.to_account_info(),
                },
            ),
            0,                                // decimals
            &self.mint_authority.key(),       // mint authority
            Some(&self.mint_authority.key()), // freeze authority
        )?;

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
