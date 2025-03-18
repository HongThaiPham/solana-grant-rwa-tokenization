# Solana Realworld asset tokenization

### Features

#### `rwa-tokenization` program:

- Initialize the program with config.
- Issue `Minter NFT` to grant `Minter` permission to user. Only `Minter` can mint tokens.
- Issue `Consumer NFT` to grant `Consumer` permission to user. Only `Consumer` can exchange tokens.
- Update quota credits for `Minter`.
- Allow use with `Minter NFT` to create a new `Token` and mint more tokens based on the credits available.
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

Program initialized: https://explorer.solana.com/tx/4YEgUYQt3eLxEq7PvTph1DfUnv3GahecQS4kKC9BQzKjBWZfPwN3oFeQaoxkfCgs3SRnviCfi8op7AvwFYQg13e?cluster=devnet

---

Issue minter cert nft

NFT minted: https://explorer.solana.com/tx/4bzo2AW45EJiHA8UENWLkwDBFWiusR98WXeFrXzwtNQC2vRhbTM4DfFRFXJzmLoznJCMtuNauhrkHfqa46zkL57H?cluster=devnet

---

Update quota credits for minter nft to 1000

Quota credits updated: https://explorer.solana.com/tx/3nmQP3dbMQda91u2UQcd4Gwk2UzVRvGFyxovSLbbC2WkzZPioTuWWWHK9gbpa3y4hoWTpCDy84uo8N9YoiMcen9s?cluster=devnet

---

Issue consumer cert nft

Consumer NFT minted tx: https://explorer.solana.com/tx/3pPxo8pPNufdqCtkKz1AwGWreoaweGXnvWWU3r8UkJnW46FtCmqQgpdaRKyoZVZZesaUpjxUXCgsGMEiSW2axjYE?cluster=devnet

---

Init token carbon credits mint

Token carbon credits mint initialized: https://explorer.solana.com/tx/hvmdbbF9FJUT8krn8tgQotAkhT3Xc7D6rofutvcfa1ZXAouTmumvDKE52ryPqZPZH6tzb6cLTh27ogEhUPxQeUg?cluster=devnet

---

Mint more carbon credits token

Token carbon credits minted: https://explorer.solana.com/tx/48m63edUpYq3xtNiZZMxPXkfNNxjXca4s49tBgFpEVPrKACcVsnDF6xSvNQaC13ECCyJNQPP1WpJKFUwVmTV346r?cluster=devnet

---

Create associated token account for receiver

Token carbon credits associated account created: https://explorer.solana.com/tx/2NynSQQds4VJkiKp8cVY6Fgjbdxko4kSPuRxEsUjQb3Lnc8vEfrJ4Tg1afoiF3dzKsCJnfMj4E5duPzcXmEuLuLS?cluster=devnet

---

Issue consumer cert nft for minter to transfer

Consumer NFT minted tx: https://explorer.solana.com/tx/2YDjq65TVRuMdX6XQy91HX3i3mUTgpvdcGWoix4YscuUnQNwHuxTZLx1ZwuA5WrDzwHwcGvW1A7dxam4UvEEdCmq?cluster=devnet

---

Transfer 10 carbon credits token from minter to receiver

Token carbon credits transferred: https://explorer.solana.com/tx/23CSbqnEq8ZaR4GZc8EwRwxYTMZQrLLgGY5j6j3hgAph933c3qYxubtS5wqJyDmWbWXgJXC7odiiyL5wFKe2v2c4?cluster=devnet

---

Retire 10 carbon credits token from consumer and receive nft certificate

Consumer NFT minted tx: https://explorer.solana.com/tx/atEDPWstGx8bCeQfwgXgfdK8i2n5kQ65ijv9dcdBk434gTDxobrpME9jKYWNa1MgGy8diJWTkLGJ9MkTApTZe7y?cluster=devnet
