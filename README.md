# Solana Realworld asset tokenization

### Features

#### `rwa-tokenization` program:

- Initialize the program with config.
- Admin create some RWA token mint.
- Issue `Minter NFT` to grant `Minter` permission to user. Only `Minter` can mint tokens.
- Issue `Consumer NFT` to grant `Consumer` permission to user. Only `Consumer` can exchange tokens.
- Update quota credits for `Minter`.
- Allow user with `Minter NFT` can mint more tokens based on the credits available.
- Allow `Consumer` retire tokens and get certificate nft.

#### `token-transfer-hook` [rpgram]:

- Validate `Consumer NFT` before token transfer.

## How to run

- Build project:

```bash
anchor build
anchor keys sync
anchor build
```

- Build and deploy `token-transfer-hook` program:

```bash
pnpm run build-and-deploy-hook
```

- Build and deploy `rwa-tokenization` program:

```bash
pnpm run build-and-deploy-rwa
```

- Run tests:

```bash
pnpm run test:rwa
```

## Results

Init governance config account

# Program initialized: https://explorer.solana.com/tx/5Xyuq4bMUL79dDZok8RJUdiX6YVz2H7VuEPYeqJ2YFmskkZEfewznJYHkJJU3JsaashXQKKvyzdWnrUDaWoqrSnp?cluster=devnet

Do the test with token close and has fee

---

Init token carbon credits mint

## Token carbon credits mint initialized: https://explorer.solana.com/tx/HVEeNf1UoZFW25vy8UUDmuCr5QQ4carXWwjTGsfASwwbYXbXNn6fzUwM6Qy1KZrzHYtwineUe77eeuMb11TLfzs?cluster=devnet

Issue minter cert nft

## NFT minted: https://explorer.solana.com/tx/3a2V2BYi8gjW6RAZtjmr9BNWYB4FFV1AtTCwzwHyGoeVTe9JUaoYSy5XzqSYPvsjxNbHM1vCr3e6c4TBMQa5Eqor?cluster=devnet

Update quota credits for minter nft to 1000

## Quota credits updated: https://explorer.solana.com/tx/3e96Y73u8FNDHfiQbA5pwcabLrxNjG4t9JNRZqN3PmUFUJsT3GYcgpUD6ret4d59CukJXaFZ5dPdiLA1zberXdma?cluster=devnet

Issue consumer cert nft

## Consumer NFT minted tx: https://explorer.solana.com/tx/3xpceJiHu3br5eaJKmWVj3PYo47bLDDXLAEGBN6x1TLziVnpWWNNeHN67g7bmCGL8m9537mEYueCSNrGyWPwD7uf?cluster=devnet

Mint more carbon credits token

## Token carbon credits minted: https://explorer.solana.com/tx/2awGK4CLzYpveePaak2nH8UEfLnF75yXcF1GcJEE1tRqpmcP9jWxqkQVmULwCphY9BCYKYGQsBmrf4yg7ekbYi1s?cluster=devnet

Create associated token account for receiver

## Token carbon credits associated account created: https://explorer.solana.com/tx/3FqbgJfDPnaAEnb7LmR8t1xrrFfgrcMh82KCWwwQwzntiVRKxog2hQQsVaqqbPBuTgp3RVUMqdAJC1Rd3rrdGQBM?cluster=devnet

Issue consumer cert nft for minter to transfer

## Consumer NFT minted tx: https://explorer.solana.com/tx/35kCsRjruAzgfAWM1ZDVFf7Vb4U91D2ePyfqNavy9GjbfdxtbG3Gpz5XK8zT7DZXFcUVMd9e3S4RPMNPPBUrZQ2d?cluster=devnet

Transfer 10 carbon credits token from minter to receiver

## Token carbon credits transferred: https://explorer.solana.com/tx/2wJoUDMQ25to4MjW14fEsTCTPpZsxoz81Hj1eo2Rb26Wk7szGJhYdeL1QtiKeSKxfe4mdnSr8Dax19KjbAgb9FJs?cluster=devnet

Retire 5 carbon credits token from consumer and receive nft certificate

Consumer NFT minted tx: https://explorer.solana.com/tx/59Fx81pzZxV5xefathYVdgJ1H2StpZB6cUHFuNZNSruWQpuVTjVNy6JFo63Uq3WP1GtaMBCSNJcF1CXQMhVb44Gb?cluster=devnet
