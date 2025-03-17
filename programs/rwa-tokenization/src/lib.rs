pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3Eobnrs1AS1m1cct7qRYVbEAsZax4eiU2ZdJSDw7d6tK");

#[program]
pub mod rwa_tokenization {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
