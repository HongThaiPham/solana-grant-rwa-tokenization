use anchor_lang::{prelude::*, system_program};

pub fn update_account_minimun_lamports<'info>(
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
