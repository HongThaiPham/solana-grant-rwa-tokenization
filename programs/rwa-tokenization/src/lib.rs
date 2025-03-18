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

declare_id!("4UWCg1wDjVJa2v6SczsPmg162wx4V2ebpPJFUvgrgrNc");

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
    ) -> Result<()> {
        ctx.accounts.handler(name, symbol, uri, &ctx.bumps)
    }

    pub fn mint_carbon_token(ctx: Context<MintCarbonToken>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }
}
