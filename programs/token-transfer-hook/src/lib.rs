use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        self,
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensions, StateWithExtensions,
        },
    },
    token_interface::{Mint, TokenAccount},
};
use spl_discriminator::discriminator::SplDiscriminate;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::{
    error::TransferHookError,
    instruction::{
        ExecuteInstruction, InitializeExtraAccountMetaListInstruction, TransferHookInstruction,
    },
};

#[error_code]
pub enum TokenTransferHookError {
    #[msg("You are not authorized to perform this action")]
    UnAuthorized,
}

declare_id!("3iSipiR8nmukvNan7ZWDJ2Cx7V7EmHPXLkQmsN1nrEna");

#[program]
pub mod token_transfer_hook {
    use super::*;

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        let extra_account_metas =
            InitializeExtraAccountMetaList::extra_account_metas(ctx.accounts.rwa_program.key())?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn execute(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        let source_account = &ctx.accounts.source_token;
        let destination_account = &ctx.accounts.destination_token;

        check_token_account_is_transferring(&source_account.to_account_info().try_borrow_data()?)?;
        check_token_account_is_transferring(
            &destination_account.to_account_info().try_borrow_data()?,
        )?;

        msg!("Transferring {} tokens", amount);

        let data = ctx.accounts.extra_account_meta_list.try_borrow_data()?;
        ExtraAccountMetaList::check_account_infos::<ExecuteInstruction>(
            &ctx.accounts.to_account_infos(),
            &TransferHookInstruction::Execute { amount }.pack(),
            &ctx.program_id,
            &data,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas(rwa_program.key())?.len()
        )?,
        payer = payer
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Program id issue the certificate nft
    pub rwa_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// Define extra account metas to store on extra_account_meta_list account
// In this example there are none
impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas(rwa_program: Pubkey) -> Result<Vec<ExtraAccountMeta>> {
        let account_metas = vec![
            ExtraAccountMeta::new_with_pubkey(&rwa_program, false, false)?,
            ExtraAccountMeta::new_external_pda_with_seeds(
                5,
                &[
                    Seed::Literal {
                        bytes: b"c".to_vec(),
                    },
                    Seed::AccountKey { index: 1 },
                    Seed::AccountData {
                        account_index: 0,
                        data_index: 32,
                        length: 32,
                    },
                ],
                false,
                false,
            )?,
            ExtraAccountMeta::new_external_pda_with_seeds(
                5,
                &[
                    Seed::Literal {
                        bytes: b"c".to_vec(),
                    },
                    Seed::AccountKey { index: 1 },
                    Seed::AccountData {
                        account_index: 2,
                        data_index: 32,
                        length: 32,
                    },
                ],
                false,
                false,
            )?,
        ];
        Ok(account_metas)
    }
}

#[derive(Accounts)]
pub struct TransferHook<'info> {
    // index 0-3 are the accounts required for token transfer (source, mint, destination, owner)
    #[account(token::mint = mint, token::authority = owner)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,
    // index 4 is address of ExtraAccountMetaList account
    // The `addExtraAccountsToInstruction` JS helper function resolving incorrectly
    /// CHECK: ExtraAccountMetaList Account,
    #[account(seeds = [b"extra-account-metas", mint.key().as_ref()], bump)]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: Program id issue the certificate nft
    pub rwa_program: AccountInfo<'info>,
    #[account(
        constraint = source_mint_nft.decimals == 0,
        constraint = source_mint_nft.supply == 1 @TokenTransferHookError::UnAuthorized,
        seeds = [b"c", mint.key().as_ref(), source_token.owner.as_ref()],
        bump,
        seeds::program = rwa_program,
    )]
    pub source_mint_nft: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        constraint = destination_mint_nft.decimals == 0,
        constraint = destination_mint_nft.supply == 1 @TokenTransferHookError::UnAuthorized,
        seeds = [b"c", mint.key().as_ref(), destination_token.owner.as_ref()],
        bump,
        seeds::program = rwa_program,
    )]
    pub destination_mint_nft: Box<InterfaceAccount<'info, Mint>>,
}

fn check_token_account_is_transferring(account_data: &[u8]) -> Result<()> {
    let token_account =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(account_data)?;
    let extension: &TransferHookAccount = token_account.get_extension::<TransferHookAccount>()?;
    if bool::from(extension.transferring) {
        Ok(())
    } else {
        Err(Into::<ProgramError>::into(
            TransferHookError::ProgramCalledOutsideOfTransfer,
        ))?
    }
}
