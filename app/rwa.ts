import { BN, Program, web3 } from "@coral-xyz/anchor";
import { explorerUrl, getConfig } from "./helpers";
import { RwaTokenization } from "../target/types/rwa_tokenization";
import * as idlRwaTokenization from "../target/idl/rwa_tokenization.json";
import { TokenTransferHook } from "../target/types/token_transfer_hook";
import * as idlTokenTransferHook from "../target/idl/token_transfer_hook.json";
import {
  address,
  addSignersToTransactionMessage,
  appendTransactionMessageInstruction,
  createTransactionMessage,
  generateKeyPairSigner,
  getAddressEncoder,
  getProgramDerivedAddress,
  getSignatureFromTransaction,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import {
  fromLegacyPublicKey,
  fromLegacyTransactionInstruction,
} from "@solana/compat";
import {
  createTransferCheckedWithTransferHookInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import {
  getCreateAssociatedTokenInstruction,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";

(async () => {
  const {
    payer,
    rpc,
    sendAndConfirmTransaction,
    provider,
    minternftMetadata,
    consumernftMetadata,
    tokenUri,
    connection,
  } = await getConfig();
  const addressEncoder = getAddressEncoder();
  const program = new Program<RwaTokenization>(idlRwaTokenization, provider);
  const admin = payer;

  const minter = await generateKeyPairSigner();
  const consumer1 = await generateKeyPairSigner();
  const transferHookProgram = idlTokenTransferHook.address;

  const [governanceConfigAccount] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("config")],
  });

  const governanceConfigAccountData = await rpc
    .getAccountInfo(governanceConfigAccount)
    .send();
  if (governanceConfigAccountData.value === null) {
    console.info("Init governance config account");
    // init governance program
    {
      let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

      const initializeProgram = await program.methods
        .initialize()
        .accounts({
          singer: new web3.PublicKey(admin.address),
        })
        .instruction();

      const transactionMintNftMessage = pipe(
        createTransactionMessage({
          version: 0,
        }),
        (tx) => setTransactionMessageFeePayer(admin.address, tx),

        (tx) =>
          setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) =>
          appendTransactionMessageInstruction(
            fromLegacyTransactionInstruction(initializeProgram),
            tx
          ),
        (tx) => addSignersToTransactionMessage([payer], tx)
      );

      const signedTransactionMintNft = await signTransactionMessageWithSigners(
        transactionMintNftMessage
      );

      await sendAndConfirmTransaction(signedTransactionMintNft, {
        commitment: "confirmed",
      });

      console.info(
        `Program initialized: ${explorerUrl(
          getSignatureFromTransaction(signedTransactionMintNft)
        )}`
      );
    }
  }

  // issue a minter cert nft
  {
    console.info("Issue minter cert nft");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueMinterCert(
        minternftMetadata.name,
        minternftMetadata.symbol,
        minternftMetadata.uri
      )
      .accounts({
        receiver: minter.address,
      })
      .instruction();

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        appendTransactionMessageInstruction(
          fromLegacyTransactionInstruction(issueNftInstruction),
          tx
        ),
      (tx) => addSignersToTransactionMessage([admin], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `NFT minted: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // update quota credits for minter nft
  {
    console.info("Update quota credits for minter nft to 1000");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const updateQuotaCreditsInstruction = await program.methods
      .updateQuotaCredit(new BN(1000))
      .accounts({
        receiver: minter.address,
      })
      .instruction();

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        appendTransactionMessageInstruction(
          fromLegacyTransactionInstruction(updateQuotaCreditsInstruction),
          tx
        ),
      (tx) => addSignersToTransactionMessage([admin], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `Quota credits updated: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // issue a consumer cert nft
  {
    console.info("Issue consumer cert nft");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueConsumerCert(
        consumernftMetadata.name,
        consumernftMetadata.symbol,
        consumernftMetadata.uri
      )
      .accounts({
        receiver: consumer1.address,
      })
      .instruction();

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        appendTransactionMessageInstruction(
          fromLegacyTransactionInstruction(issueNftInstruction),
          tx
        ),
      (tx) => addSignersToTransactionMessage([admin], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `Consumer NFT minted tx: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // init token carbon credits mint
  {
    console.info("Init token carbon credits mint");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const initializeTokenMint = await program.methods
      .initCarbonToken("Carbon Credits", "CC", tokenUri)
      .accounts({
        payer: admin.address,
        creator: minter.address,
        transferHookProgram,
      })
      .instruction();

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        appendTransactionMessageInstruction(
          fromLegacyTransactionInstruction(initializeTokenMint),
          tx
        ),
      (tx) => addSignersToTransactionMessage([admin, minter], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `Token carbon credits mint initialized: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // mint more carbon credits token
  {
    console.info("Mint more carbon credits token");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const mintTokenInstruction = await program.methods
      .mintCarbonToken(new BN(300))
      .accounts({
        creator: minter.address,
        payer: admin.address,
        transferHookProgram,
      })
      .instruction();

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        appendTransactionMessageInstruction(
          fromLegacyTransactionInstruction(mintTokenInstruction),
          tx
        ),
      (tx) => addSignersToTransactionMessage([admin, minter], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `Token carbon credits minted: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // try to transfer carbon credits token
  {
    const [nftMinterMintAddress] = await getProgramDerivedAddress({
      programAddress: fromLegacyPublicKey(program.programId),
      seeds: [Buffer.from("m"), addressEncoder.encode(minter.address)],
    });

    const [carbonCreditsMintAddress] = await getProgramDerivedAddress({
      programAddress: fromLegacyPublicKey(program.programId),
      seeds: [Buffer.from("cct"), addressEncoder.encode(nftMinterMintAddress)],
    });

    const senderAta = getAssociatedTokenAddressSync(
      new web3.PublicKey(carbonCreditsMintAddress),
      new web3.PublicKey(minter.address),
      false,
      TOKEN_2022_PROGRAM_ID
    );

    const receiverAta = getAssociatedTokenAddressSync(
      new web3.PublicKey(carbonCreditsMintAddress),
      new web3.PublicKey(consumer1.address),
      false,
      TOKEN_2022_PROGRAM_ID
    );

    {
      console.log("Create associated token account for receiver");
      let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
      const transaction = pipe(
        createTransactionMessage({
          version: 0,
        }),
        (tx) => setTransactionMessageFeePayer(admin.address, tx),
        (tx) =>
          setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) =>
          appendTransactionMessageInstruction(
            getCreateAssociatedTokenInstruction({
              payer: payer,
              mint: carbonCreditsMintAddress,
              tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
              owner: consumer1.address,
              ata: address(receiverAta.toString()),
            }),
            tx
          ),
        (tx) => addSignersToTransactionMessage([admin], tx)
      );

      const signedTransaction = await signTransactionMessageWithSigners(
        transaction
      );

      await sendAndConfirmTransaction(signedTransaction, {
        commitment: "confirmed",
      });

      console.info(
        `Token carbon credits associated account created: ${explorerUrl(
          getSignatureFromTransaction(signedTransaction)
        )}`
      );
    }

    {
      console.info("Transfer 10 carbon credits token from minter to receiver");
      const ix = await createTransferCheckedWithTransferHookInstruction(
        connection,
        senderAta,
        new web3.PublicKey(carbonCreditsMintAddress),
        receiverAta,
        new web3.PublicKey(minter.address),
        BigInt(10),
        0,
        [],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

      let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
      const transaction = pipe(
        createTransactionMessage({
          version: 0,
        }),
        (tx) => setTransactionMessageFeePayer(admin.address, tx),
        (tx) =>
          setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) =>
          appendTransactionMessageInstruction(
            fromLegacyTransactionInstruction(ix),
            tx
          ),
        (tx) => addSignersToTransactionMessage([admin, minter], tx)
      );

      const signedTransaction = await signTransactionMessageWithSigners(
        transaction
      );

      await sendAndConfirmTransaction(signedTransaction, {
        commitment: "confirmed",
      });

      console.info(
        `Token carbon credits transferred: ${explorerUrl(
          getSignatureFromTransaction(signedTransaction)
        )}`
      );
    }
  }
})();
