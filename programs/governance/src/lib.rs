pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("AZXWgDHioQgyB1FuvmwGp5QadJJdmuYz15K9qid74d7M");

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
}
