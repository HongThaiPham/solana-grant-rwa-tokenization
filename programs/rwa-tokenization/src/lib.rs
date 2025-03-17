pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("E1i3Azc7n7pvb3KGecb7B9nZANRuWeZvf27eBsa2sQ8d");

#[program]
pub mod rwa_tokenization {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
