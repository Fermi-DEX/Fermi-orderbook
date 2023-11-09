import * as anchor from '@project-serum/anchor';
import Provider from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { assert } from 'chai';
import { SimpleSerum } from '../../target/types/fermi_dex';
import idl from "../../target/idl/fermi_dex.json";
import solblog_keypair from "/Users/dm/Documents/blob_solana/wallet/fermi-orderbook/target/deploy/fermi_dex-keypair.json"
const fs = require('fs');
import { Token } from '@solana/spl-token';
import { Connection, PublicKey } from '@solana/web3.js';
import {
    asksPda,
    bidsPda,
    coinMint,
    coinVault,
    eventQPda,
    marketPda,
    pcMint,
    pcVault,
    reqQPda,
    programId,
  } from "./utils/consts_nov9_2";

const {Keypair} = require("@solana/web3.js");
const secretKey = JSON.parse(fs.readFileSync("/Users/dm/.config/solana/id.json"));
const secretKeynew = JSON.parse(fs.readFileSync("/Users/dm/Documents/fermi_labs/basic/keypair2/keypair2.json"));

const secretKeySecond = JSON.parse(fs.readFileSync("./kp3/key.json"));

const keypair = Keypair.fromSecretKey(new Uint8Array(secretKeySecond));
//const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
//const keypair = anchor.web3.Keypair.generate();

const authority = keypair;

let openOrdersPda: anchor.web3.PublicKey;
let openOrdersPdaBump: number;
async function fetchTokenBalance(mintAddress: string, userAddress: string) {
    const connection = new Connection('http://localhost:8899');
    
    const mintPublicKey = new PublicKey(mintAddress);
    const userPublicKey = new PublicKey(userAddress);
  
    let tokenBalance = await connection.getTokenAccountBalance(userPublicKey);
  
    console.log('Token balance:', tokenBalance);
    return tokenBalance;
  }

console.log('testing new bid keypair');
async function fetchTokenBalance2(mintAddress: string, associatedTokenAddress: string) {
    // Get the connection from the provider
    const provider = anchor.AnchorProvider.env();

    const connection = provider.connection;
  
    async function fetchTokenBalance(mintAddress: string, associatedTokenAddress: string) {
        // Create PublicKey objects for the mint and associated token address
        const mintPublicKey = new PublicKey(mintAddress);
        const associatedTokenPublicKey = new PublicKey(associatedTokenAddress);
      
        // Create a Token object for the token
        const token = new Token(provider.connection, mintPublicKey, spl.TOKEN_PROGRAM_ID, provider.wallet.payer);
      
        // Fetch the associated token account info
        const tokenAccountInfo = await token.getAccountInfo(associatedTokenPublicKey);
      
        // Log the balance
        console.log('Token balance:', tokenAccountInfo.amount.toString());
      }
  }
  /*
async function fetchTokenBalance(tokenMintAddress: string, userPublicKey: string) {
    // Create a PublicKey object for the user's address
    const userAddress = new PublicKey(userPublicKey);
  
    // Create a PublicKey object for the token's mint address
    const mintAddress = new PublicKey(tokenMintAddress);
  
    // Create a Token object for the token
    const token = new Token(connection, mintAddress, {}, {});
  
    // Fetch the user's token account info
    const tokenAccountInfo = await token.getAccountInfo(userAddress);
  
    // Log the balance
    console.log('Token balance:', tokenAccountInfo.amount.toString());
  }*/

describe('fermi-dex-new', () => {
    before(async () => {

       

    });
describe('#new_order', async () => {
    it('New order - sell @ 19 successful', async () => {
        console.log('testing new ask')
      {
        const provider = anchor.AnchorProvider.env();

        /*
        const rpcUrl = 'https://api.devnet.solana.com';  // You can replace this with the appropriate RPC URL for your network.
        const rpcUrlLocal = 'http://localhost:8899';
        const wallet = new anchor.Wallet(keypair);
        const conn = new Connection(rpcUrlLocal);
        const provider = new anchor.AnchorProvider(conn, wallet, anchor.AnchorProvider.defaultOptions());*/

        //const program = new anchor.Program(idl, programId, provider) //for existing prog
/*
            const rpcUrl = 'https://api.devnet.solana.com';  // You can replace this with the appropriate RPC URL for your network.
            const provider = new anchor.Provider(rpcUrl, keypair, {
      preflightCommitment: 'recent',
      commitment: 'confirmed',
    }); */

        const program = new anchor.Program(idl, programId, provider) //for existing prog
        const authorityPcTokenAccount = await spl.getAssociatedTokenAddress(
            new anchor.web3.PublicKey(pcMint),
            authority.publicKey,
            false,
          );
        const authorityCoinTokenAccount = await spl.getAssociatedTokenAddress(
            new anchor.web3.PublicKey(coinMint),
            authority.publicKey,
            false,
          );
        
      
        [openOrdersPda, openOrdersPdaBump] =
          await anchor.web3.PublicKey.findProgramAddress(
            [
              Buffer.from('open-orders', 'utf-8'),
              new anchor.web3.PublicKey(marketPda).toBuffer(),
              authority.publicKey.toBuffer(),
            ],
            new anchor.web3.PublicKey(programId),
          );
          
          const objectsToCheck = {
            openOrdersPda,
            marketPda,
            pcVault,
            coinMint,
            pcMint,
            reqQPda,
            eventQPda,
            authority: authority.publicKey,
            pcpayer: authorityPcTokenAccount,
            coinpayer: authorityCoinTokenAccount,
          };
          
          for (const [key, value] of Object.entries(objectsToCheck)) {
            console.log(`${key}: Type - ${typeof value} (${value.constructor.name}), Value - ${value}`);
          }
        await program.methods
          .newOrder(
            { ask: {} },
            new anchor.BN(34),
            new anchor.BN(1),
            new anchor.BN(34),
            { limit: {} },
          )
          .accounts({
            openOrders: openOrdersPda,
            market: marketPda,
            coinVault,
            pcVault,
            coinMint: coinMint,
            pcMint: pcMint,
            payer: authorityCoinTokenAccount,
            bids: bidsPda,
            asks: asksPda,
            reqQ: reqQPda,
            eventQ: eventQPda,
            authority: authority.publicKey,
          })
          .signers([authority])
          .rpc();  

        console.log('place limit order sell price: 22');
      
        const openOrders = await program.account.openOrders.fetch(
          openOrdersPda,
        );
        //console.log(openOrders);
        const bids = await program.account.orders.fetch(bidsPda);
        //console.log(bids);
        const asks = await program.account.orders.fetch(asksPda);
        //console.log(asks);
        const eventQ = await program.account.eventQueue.fetch(eventQPda);
        console.log(eventQ);
        const pcbal = await fetchTokenBalance(pcMint, authorityPcTokenAccount.toString());
        const coinbal = await fetchTokenBalance(coinMint, authorityCoinTokenAccount.toString());
        console.log("Ask placed at price: 25 successful");
        console.log("PC token balance: {}", pcbal);  ;
        console.log("Coin token balance: {}", coinbal);  ; 

        let askslist = await program.account.orders.fetch(asksPda);
        console.log(askslist);
        
/*
        console.log("cancelling order");
        //const program = new anchor.Program(idl, programId, provider);

        const orderId = new anchor.BN('627189298506124754945'); // Adjust accordingly
        const expectedOwner = authority.publicKey; // Adjust accordingly
    
        await program.rpc.cancelAsk(orderId, expectedOwner, {
          accounts: {
            market: marketPda,
            bids: bidsPda,
            asks: asksPda,
            eventQ: eventQPda,
            authority: authority.publicKey, // Assuming this is the expected owner
          },
          signers: [authority],
        });
        console.log("Ask cancelled");
        let askslist2 = await program.account.orders.fetch(asksPda);
        console.log(askslist2); */
      }
    })
    })
});