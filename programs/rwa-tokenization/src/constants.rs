use anchor_lang::prelude::*;

#[constant]
pub const GOVERNANCE_CONFIG_SEED: &[u8] = b"config";

pub const MINTER_NFT_SEED: &[u8] = b"m"; // minter
pub const CONSUMER_NFT_SEED: &[u8] = b"c"; // consumer
pub const MINT_AUTHORITY_SEED: &[u8] = b"ma"; // mint authority
pub const CARBON_CREDIT_TOKEN_SEED: &[u8] = b"cct"; // carbon credit token

pub static AVAILABLE_CREDITS_KEY: &str = "available_credits";
pub static MINTED_CREDITS_KEY: &str = "minted_credits";
