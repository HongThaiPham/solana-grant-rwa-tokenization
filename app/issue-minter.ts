import { BN, Program, web3 } from "@coral-xyz/anchor";
import { getConfig } from "./helpers";
import { Governance } from "../target/types/governance";
import * as idlGovernance from "../target/idl/governance.json";
import {
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
import { fromLegacyPublicKey, fromLegacyTransactionInstruction } from "@solana/compat";

(async () => {
  const {
    payer,
    rpc,
    sendAndConfirmTransaction,
    provider,
    minternftMetadata,
    consumernftMetadata,
    tokenUri
  } = await getConfig();
  const addressEncoder = getAddressEncoder();
  const program = new Program<Governance>(idlGovernance, provider);
  const admin = payer;


  const minter = await generateKeyPairSigner();
  const consumer1 = await generateKeyPairSigner();
  const transferHookProgram = web3.Keypair.generate().publicKey;

  const [governanceConfigAccount] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("config")],
  })

  const governanceConfigAccountData = await rpc.getAccountInfo(governanceConfigAccount).send();
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

        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
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
        `Program initialized: ${getSignatureFromTransaction(
          signedTransactionMintNft
        )}`
      );
    }
  }

  // issue a minter cert nft
  {
    console.info("Issue minter cert nft");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueMinterCert(minternftMetadata.name, minternftMetadata.symbol, minternftMetadata.uri)
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
      `NFT minted: ${getSignatureFromTransaction(signedTransactionMintNft)}`
    );
  }


  const [minterNftAccount] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("minter"), addressEncoder.encode(minter.address)],
  })


  // update quota credits for minter nft
  {
    console.info("Update quota credits for minter nft to 1000");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const updateQuotaCreditsInstruction = await program.methods
      .updateQuotaCredit(new BN(1000))
      .accounts({
        receiver: minter.address
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
      `Quota credits updated: ${getSignatureFromTransaction(
        signedTransactionMintNft
      )}`
    );
  }

  // issue a consumer cert nft
  {
    console.info("Issue consumer cert nft");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueConsumerCert(consumernftMetadata.name, consumernftMetadata.symbol, consumernftMetadata.uri)
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
      `Consumer NFT minted tx: ${getSignatureFromTransaction(signedTransactionMintNft)}`
    );
  }

  // init token carbon credits mint
  const carbonCreditMint = await generateKeyPairSigner();
  {
    console.info("Init token carbon credits mint");
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const initializeTokenMint = await program.methods
      .initCarbonToken("Carbon Credits", "CC", tokenUri)
      .accounts({
        payer: admin.address,
        creator: minter.address,
        mint: carbonCreditMint.address,
        transferHookProgram
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
      (tx) => addSignersToTransactionMessage([admin, minter, carbonCreditMint], tx)
    );

    const signedTransactionMintNft = await signTransactionMessageWithSigners(
      transactionMintNftMessage
    );

    await sendAndConfirmTransaction(signedTransactionMintNft, {
      commitment: "confirmed",
    });

    console.info(
      `Token carbon credits mint initialized: ${getSignatureFromTransaction(
        signedTransactionMintNft
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
        mint: carbonCreditMint.address,
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
      `Token carbon credits minted: ${getSignatureFromTransaction(
        signedTransactionMintNft
      )}`
    );
  }
})();
