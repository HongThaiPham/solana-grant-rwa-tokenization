pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("pJ6N5doswUVH9jXgkZKJtgubofPKwv18AAYxSRiB68g");

#[program]
pub mod rwa_tokenization {
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

    pub fn update_quota_credit(ctx: Context<UpdateQuotaCredit>, new_credit: u64) -> Result<()> {
        ctx.accounts.handler(new_credit)
    }

    pub fn issue_consumer_cert(
        ctx: Context<IssueConsumerCert>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.handler(name, symbol, uri)
    }

    pub fn init_carbon_token(
        ctx: Context<InitCarbonToken>,
        name: String,
        symbol: String,
        uri: String,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
    ) -> Result<()> {
        ctx.accounts.handler(
            name,
            symbol,
            uri,
            transfer_fee_basis_points,
            maximum_fee,
            &ctx.bumps,
        )
    }

    pub fn mint_carbon_token(ctx: Context<MintCarbonToken>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn retire_token(ctx: Context<RetireToken>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }
}
