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
  appendTransactionMessageInstructions,
  createTransactionMessage,
  generateKeyPairSigner,
  getAddressEncoder,
  getProgramDerivedAddress,
  getSignatureFromTransaction,
  KeyPairSigner,
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
    payer: admin,
    rpc,
    sendAndConfirmTransaction,
    provider,
  } = await getConfig();
  const program = new Program<RwaTokenization>(idlRwaTokenization, provider);

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
        (tx) => addSignersToTransactionMessage([admin], tx)
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

  await test_token_close_and_has_fee();
  // await test_token_open_and_has_fee();
  // await test_token_close_and_no_fee();
  // await test_token_open_and_no_fee();
})();

const test_token_close_and_no_fee = async () => {
  console.info("==============================================");
  console.info("Do the test with token close and no fee");
  await do_test(`CC${Math.floor(Math.random() * 10000)}`, true, false);
  console.info("Do the test with token close and no fee - Test completed");
};
const test_token_open_and_no_fee = async () => {
  console.info("==============================================");
  console.info("Do the test with token open and no fee");
  await do_test(`CC${Math.floor(Math.random() * 10000)}`, false, false);
  console.info("Do the test with token open and no fee - Test completed");
};

const test_token_open_and_has_fee = async () => {
  console.info("==============================================");
  console.info("Do the test with token open and has fee");
  // Fee basis points for ctransfers (100 = 1%)
  const feeBasisPoints = 100;
  // Maximum fee for transfers in token base units
  const maxFee = new BN(100);
  await do_test(
    `CC${Math.floor(Math.random() * 10000)}`,
    false,
    true,
    feeBasisPoints,
    maxFee
  );
  console.info("Do the test with token open and has fee - Test completed");
};
const test_token_close_and_has_fee = async () => {
  console.info("==============================================");
  console.info("Do the test with token close and has fee");
  // Fee basis points for transfers (100 = 1%)
  const feeBasisPoints = 100;
  // Maximum fee for transfers in token base units
  const maxFee = new BN(100);
  await do_test(
    `CC${Math.floor(Math.random() * 10000)}`,
    true,
    true,
    feeBasisPoints,
    maxFee
  );
  console.info("Do the test with token close and has fee - Test completed");
};

const do_test = async (
  symbol: string = "CC",
  isClose: boolean,
  hasFee: boolean,
  feeBasisPoints?: number,
  maxFee?: BN
) => {
  const decimals = 9;
  const minter = await generateKeyPairSigner();
  const consumer1 = await generateKeyPairSigner();
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
  const transferHookProgram = new Program<TokenTransferHook>(
    idlTokenTransferHook,
    provider
  );
  const admin = payer;

  // init token carbon credits mint
  const [nftMinterMintAddress] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("m"), addressEncoder.encode(minter.address)],
  });

  const [carbonCreditsMintAddress] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("cct"), Buffer.from(symbol)],
  });

  {
    console.log("--------------------");
    console.info("Init token carbon credits mint");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const initializeTokenMint = await program.methods
      .initRwaToken(
        "Carbon Credits",
        symbol,
        decimals,
        tokenUri,
        isClose,
        hasFee,
        feeBasisPoints ?? 0,
        maxFee ?? new BN(0)
      )
      .accounts({
        transferHookProgram: transferHookProgram.programId,
      })
      .instruction();
    let initializeExtraAccountMetaListInstruction;
    if (isClose) {
      initializeExtraAccountMetaListInstruction =
        await transferHookProgram.methods
          .initializeExtraAccountMetaList()
          .accounts({
            payer: payer.address,
            mint: carbonCreditsMintAddress,
            rwaProgram: program.programId,
          })
          .instruction();
    }

    const transactionMintNftMessage = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (tx) => setTransactionMessageFeePayer(admin.address, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) =>
        !isClose
          ? appendTransactionMessageInstruction(
              fromLegacyTransactionInstruction(initializeTokenMint),
              tx
            )
          : appendTransactionMessageInstructions(
              [
                fromLegacyTransactionInstruction(initializeTokenMint),
                fromLegacyTransactionInstruction(
                  initializeExtraAccountMetaListInstruction
                ),
              ],
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
      `Token carbon credits mint initialized: ${explorerUrl(
        getSignatureFromTransaction(signedTransactionMintNft)
      )}`
    );
  }

  // issue a minter cert nft
  {
    console.log("--------------------");
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
        permissionedMint: carbonCreditsMintAddress,
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
    console.log("--------------------");
    console.info("Update quota credits for minter nft to 1000");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const updateQuotaCreditsInstruction = await program.methods
      .updateQuotaCredit(new BN(1000))
      .accounts({
        receiver: minter.address,
        permissionedMint: carbonCreditsMintAddress,
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
    console.log("--------------------");
    console.info("Issue consumer cert nft");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueConsumerCert(
        consumernftMetadata.name,
        consumernftMetadata.symbol,
        consumernftMetadata.uri
      )
      .accounts({
        minter: minter.address,
        payer: admin.address,
        receiver: consumer1.address,
        rwaMint: carbonCreditsMintAddress,
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
      (tx) => addSignersToTransactionMessage([admin, minter], tx)
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

  // mint more carbon credits token
  {
    console.log("--------------------");
    console.info("Mint more carbon credits token");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const mintTokenInstruction = await program.methods
      .mintRwaToken(new BN(300))
      .accounts({
        minter: minter.address,
        payer: admin.address,
        receiver: minter.address,
        rwaMint: carbonCreditsMintAddress,
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
      console.log("--------------------");
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
    await new Promise((resolve) => setTimeout(resolve, 10000));

    if (isClose) {
      console.log("--------------------");
      console.info("Issue consumer cert nft for minter to transfer");
      let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

      const instruction = await program.methods
        .issueConsumerCert(
          consumernftMetadata.name,
          consumernftMetadata.symbol,
          consumernftMetadata.uri
        )
        .accounts({
          minter: minter.address,
          payer: admin.address,
          receiver: minter.address,
          rwaMint: carbonCreditsMintAddress,
        })
        .instruction();

      const transaction = pipe(
        createTransactionMessage({
          version: 0,
        }),
        (tx) => setTransactionMessageFeePayer(admin.address, tx),
        (tx) =>
          setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) =>
          appendTransactionMessageInstruction(
            fromLegacyTransactionInstruction(instruction),
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
        `Consumer NFT minted tx: ${explorerUrl(
          getSignatureFromTransaction(signedTransaction)
        )}`
      );
    }

    // wait for init account
    await new Promise((resolve) => setTimeout(resolve, 10000));

    {
      console.log("--------------------");
      console.info("Transfer 10 carbon credits token from minter to receiver");
      const ix = await createTransferCheckedWithTransferHookInstruction(
        connection,
        senderAta,
        new web3.PublicKey(carbonCreditsMintAddress),
        receiverAta,
        new web3.PublicKey(minter.address),
        BigInt(10),
        decimals,
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

    // retire token
    {
      console.log("--------------------");
      console.info(
        "Retire 5 carbon credits token from consumer and receive nft certificate"
      );
      let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
      const nftMint = await generateKeyPairSigner();

      const instruction = await program.methods
        .retireToken(new BN(5))
        .accounts({
          payer: admin.address,
          consumer: consumer1.address,
          mint: carbonCreditsMintAddress,
          nftMint: nftMint.address,
        })
        .instruction();

      const transaction = pipe(
        createTransactionMessage({
          version: 0,
        }),
        (tx) => setTransactionMessageFeePayer(admin.address, tx),
        (tx) =>
          setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) =>
          appendTransactionMessageInstruction(
            fromLegacyTransactionInstruction(instruction),
            tx
          ),
        (tx) => addSignersToTransactionMessage([admin, consumer1, nftMint], tx)
      );

      const signedTransaction = await signTransactionMessageWithSigners(
        transaction
      );

      await sendAndConfirmTransaction(signedTransaction, {
        commitment: "confirmed",
      });

      console.info(
        `Consumer NFT minted tx: ${explorerUrl(
          getSignatureFromTransaction(signedTransaction)
        )}`
      );
    }
  }
};
