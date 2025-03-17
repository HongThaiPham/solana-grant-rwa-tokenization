import { Program, web3 } from "@coral-xyz/anchor";
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
    payerPublicKey,
    rpc,
    sendAndConfirmTransaction,
    collectionMetadata,
    provider,
    nftMetadata,
  } = await getConfig();
  const program = new Program<Governance>(idlGovernance, provider);
  const admin = payer;

  const recevier = await generateKeyPairSigner();

  const nftMint = await generateKeyPairSigner();

  const addressEncoder = getAddressEncoder();
  const [governanceConfigAccount] = await getProgramDerivedAddress({
    programAddress: fromLegacyPublicKey(program.programId),
    seeds: [Buffer.from("config")],
  })

  const governanceConfigAccountData = await rpc.getAccountInfo(governanceConfigAccount);
  if (governanceConfigAccountData === null) {
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
        (tx) => addSignersToTransactionMessage([payer, nftMint], tx)
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

  // mint nft
  {
    let { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const issueNftInstruction = await program.methods
      .issueMinterCert(nftMetadata.name, nftMetadata.symbol, nftMetadata.uri)
      .accounts({
        receiver: new web3.PublicKey(recevier.address),
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
})();
