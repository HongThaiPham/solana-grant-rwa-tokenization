import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  sendAndConfirmTransactionFactory,
} from "@solana/kit";

import dotenv from "dotenv";
dotenv.config();

const PAYER_PRIVATE_KEY = process.env.PAYER_PRIVATE_KEY as string;
const RPC_HOST = process.env.RPC_HOST as string;

export async function getConfig() {
  const oldPayer = web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(PAYER_PRIVATE_KEY))
  );
  const payer = await createKeyPairSignerFromBytes(
    new Uint8Array(JSON.parse(PAYER_PRIVATE_KEY))
  );

  const connection = new web3.Connection(`https://${RPC_HOST}`);
  const wallet = new NodeWallet(oldPayer);
  const payerPublicKey = new web3.PublicKey(payer.address);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "processed",
  });

  const rpc = createSolanaRpc(`https://${RPC_HOST}`);
  const rpcSubscriptions = createSolanaRpcSubscriptions(`wss://${RPC_HOST}`);

  const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
    /**
     * The RPC implements a `sendTransaction` method which relays transactions to the network.
     */
    rpc,
    /**
     * RPC subscriptions allow the transaction sender to subscribe to the status of our transaction.
     * The sender will resolve when the transaction is reported to have been confirmed, or will
     * reject in the event of an error, or a timeout if the transaction lifetime is thought to have
     * expired.
     */
    rpcSubscriptions,
  });

  const collectionMetadata = {
    name: "Carbon NFT Collection",
    symbol: "CCNFT",
    uri: "https://arweave.net/1234",
  };

  const nftMetadata = {
    name: "Minter NFT",
    symbol: "MNT",
    uri: "https://arweave.net/1234",
  };
  return {
    provider,
    connection,
    wallet,
    oldPayer,
    payer,
    payerPublicKey,
    rpc,
    sendAndConfirmTransaction,
    collectionMetadata,
    nftMetadata,
  };
}
