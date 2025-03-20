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

Program initialized: https://explorer.solana.com/tx/4ff8VUf9UvzDzdwpWCHVw4s77YPUn1N82JqgG1AV9yTsrsiiickD7K3KeVKVFr8iDz7MvPqim498LnrQycD7D7b4?cluster=devnet

---

Issue minter cert nft

NFT minted: https://explorer.solana.com/tx/2ty1hTyNsoTrikDuzgrZhFgTKS5bHEWKiaQkdwzESTWCndCoXQz3oiJYZcNcZqDw8QKqJtETxAVCeQgBWs3WDWES?cluster=devnet

---

Update quota credits for minter nft to 1000

Quota credits updated: https://explorer.solana.com/tx/btaSazFWjhDPJiRAfRfKo91xiQ7t7Luj6n5wC7GqQcNbzjMcUG5aBgY5LM4n2fDbQmDEk9yJyWLJdywJXZ3WQKJ?cluster=devnet

---

Issue consumer cert nft

Consumer NFT minted tx: https://explorer.solana.com/tx/2LQ286MubtvDScr3HmCQ7dyuAE1tmLv57ewsJi7icJsNJ8PuL79dR71WsgLUXAweYtwgcPyKwaZK74VND8Z9ntGr?cluster=devnet

---

Init token carbon credits mint

Token carbon credits mint initialized: https://explorer.solana.com/tx/4GMvHn7ic8QE5D4X5XkN3QFighEY5x1zihharaR6y3eokwFoH5AoZ4HCsHrWagFTUGqLyNKMhou4R4x7KRUi79sp?cluster=devnet

---

Mint more carbon credits token

Token carbon credits minted: https://explorer.solana.com/tx/5ejMpSiBEPLyzLzghj39W6pfCZeFEswWmbQQd655WVYKWGsPKgjxMBiQA7PMgRhZRp6dkBumCWsRQEvsGgip1DxG?cluster=devnet

---

Create associated token account for receiver

Token carbon credits associated account created: https://explorer.solana.com/tx/4fnU5m9xrh27T4dPhttzRcdzDGmV9YkWTAzQKt9rRVAixcrnmhkjrrCP7b64FRqgxgFvPyw482aHoEUCTwHR824W?cluster=devnet

---

Issue consumer cert nft for minter to transfer

Consumer NFT minted tx: https://explorer.solana.com/tx/3Jciy4TBfMEnhdpgAMpj43suF1v8b6qoQtZ5FrkcKHqa4yskWnPrFK7qZuqvkDKtRGt99XiosVc7wFtYrPZ32CF2?cluster=devnet

---

Transfer 10 carbon credits token from minter to receiver

Token carbon credits transferred: https://explorer.solana.com/tx/5AdBAmkqVusnAUL9mVu19JoKmZt7TdYdMD4TbpWsHQLa4YZaXi6HJ6bHM4ZatfbqeuUhiwvqCwfqY5aduoYtA45u?cluster=devnet

---

Retire 5 carbon credits token from consumer and receive nft certificate

Consumer NFT minted tx: https://explorer.solana.com/tx/3RMZFhnJWm6xGXRhEMphCoBR5uQQsqhCmnX9fm9zeUaSPnfNzaceRWkf7gj7zBLG1bxZXeaqFjEAnigpA4Mj8mnf?cluster=devnet
