pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("UW1LxzYAEPSwVFMMXkF9P8fsdQZTpdStKS5pvxDmhhk");

#[program]
pub mod governance {
    use super::*;

    pub fn initialize(ctx: Context<InitializeConfig>) -> Result<()> {
        ctx.accounts.handler(&ctx.bumps)
    }

    pub fn issue_minter_cert(
        ctx: Context<IssueMinterCert>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.handler(name, symbol, uri)
    }
}
