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

declare_id!("2Dj6oCznAfdaVuBbbsW1AaNNKxshz1uHa8fbhinXzkuh");

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
        ctx.accounts.handler(name, symbol, uri, &ctx.bumps)
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
        ctx.accounts.handler(name, symbol, uri, &ctx.bumps)
    }

    pub fn init_rwa_token(
        ctx: Context<InitRwaToken>,
        name: String,
        symbol: String,
        uri: String,
        is_close: bool, // whether the mint use transfer hook extension
        has_fee: bool,  // whether the mint use transfer fee extension
        transfer_fee_basis_points: Option<u16>,
        maximum_fee: Option<u64>,
    ) -> Result<()> {
        ctx.accounts.handler(
            name,
            symbol,
            uri,
            is_close,
            has_fee,
            transfer_fee_basis_points,
            maximum_fee,
            &ctx.bumps,
        )
    }

    pub fn mint_rwa_token(ctx: Context<MintRwaToken>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn retire_token(ctx: Context<RetireToken>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }
}
