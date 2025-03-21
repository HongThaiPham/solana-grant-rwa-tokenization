use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_2022::spl_token_2022::{self, extension::ExtensionType};

pub fn update_account_minimum_lamports<'info>(
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    space: usize,
) -> Result<()> {
    let lamports_required = (Rent::get()?).minimum_balance(space);

    msg!(
        "Update account size with space: {} lamports: {}",
        space as u64,
        lamports_required
    );

    system_program::transfer(
        CpiContext::new(
            system_program,
            system_program::Transfer {
                from: payer,
                to: account,
            },
        ),
        lamports_required,
    )?;
    Ok(())
}

pub fn update_account_lamports_to_minimum_balance<'info>(
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let extra_lamports = Rent::get()?.minimum_balance(account.data_len()) - account.get_lamports();
    if extra_lamports > 0 {
        system_program::transfer(
            CpiContext::new(
                system_program,
                system_program::Transfer {
                    from: payer,
                    to: account,
                },
            ),
            extra_lamports,
        )?;
    }
    Ok(())
}

pub fn get_mint_space_with_extensions(is_close: bool, has_fee: bool) -> Result<usize> {
    msg!("is_close: {}, has_fee: {}", is_close, has_fee);
    let extensions: &[ExtensionType] = match (is_close, has_fee) {
        (true, true) => &[
            ExtensionType::MetadataPointer,
            ExtensionType::MintCloseAuthority,
            ExtensionType::TransferHook,
            ExtensionType::TransferFeeConfig,
        ],
        (true, false) => &[
            ExtensionType::MetadataPointer,
            ExtensionType::MintCloseAuthority,
            ExtensionType::TransferHook,
        ],
        (false, true) => &[
            ExtensionType::MetadataPointer,
            ExtensionType::MintCloseAuthority,
            ExtensionType::TransferFeeConfig,
        ],
        (false, false) => &[
            ExtensionType::MetadataPointer,
            ExtensionType::MintCloseAuthority,
        ],
    };

    Ok(ExtensionType::try_calculate_account_len::<
        spl_token_2022::state::Mint,
    >(extensions)?)
}
