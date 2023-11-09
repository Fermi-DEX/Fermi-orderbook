import * as anchor from '@project-serum/anchor';
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
const secretKeyThird= JSON.parse(fs.readFileSync("./kp4/key.json"));
//const secretKeyThird= JSON.parse(fs.readFileSync("./kp3/key.json"));

const keypair = Keypair.fromSecretKey(new Uint8Array(secretKeyThird));
//const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));


const authority = keypair;
console.log(authority.publicKey.toString());

let openOrdersPda: anchor.web3.PublicKey;
let openOrdersPdaBump: number;
async function fetchTokenBalance(mintAddress: string, userAddress: string) {
    const connection = new Connection("https://rpc-devnet.helius.xyz/?api-key=69bea66a-a716-416b-8a45-a9c7049b0731");
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
    it('New order - buy @ 20', async () => {
        console.log('testing new bid')
      {
        const provider = anchor.AnchorProvider.env();
        /*
        const rpcUrl = 'https://api.devnet.solana.com';  // You can replace this with the appropriate RPC URL for your network.
        const rpcUrlLocal = 'http://localhost:8899';
        const wallet = new anchor.Wallet(keypair);
        const conn = new Connection(rpcUrlLocal);
        const provider = new anchor.AnchorProvider(conn, wallet, anchor.AnchorProvider.defaultOptions()); */


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
          console.log('testing new bid keypair');
        
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
            market: marketPda,
            pcVault,
            coinMint,
            pcMint,
            reqQ: reqQPda,
            eventQ: eventQPda,
            authority: authority.publicKey,
            pcpayer: authorityPcTokenAccount,
            coinpayer: authorityCoinTokenAccount,
          };

          for (const [key, value] of Object.entries(objectsToCheck)) {
            console.log(`${key}: Type - ${typeof value} (${value.constructor.name}), Value - ${value}`);
          }
          /*
        let ordid = await program.methods
          .newOrder(
            { bid: {} },
            new anchor.BN(30),
            new anchor.BN(1),
            new anchor.BN(30),
            { limit: {} },
          )
          .accounts({
            openOrders: openOrdersPda,
            market: marketPda,
            coinVault,
            pcVault,
            coinMint: coinMint,
            pcMint: pcMint,
            payer: authorityPcTokenAccount,
            bids: bidsPda,
            asks: asksPda,
            reqQ: reqQPda,
            eventQ: eventQPda,
            authority: authority.publicKey,
          })
          .signers([authority])
          .rpc(); 

        console.log('place limit order buy price: 20');
        console.log('order id is: ', ordid); */
        let bids = await program.account.orders.fetch(bidsPda);
        console.log(bids);

        console.log("cancelling order");
        //const program = new anchor.Program(idl, programId, provider);

        const orderId = new anchor.BN('645636042579834306559'); // Adjust accordingly
        const expectedOwner = authority.publicKey; // Adjust accordingly
    
        await program.rpc.cancelBid(orderId, expectedOwner, {
          accounts: {
            openOrders: openOrdersPda,
            market: marketPda,
            bids: bidsPda,
            asks: asksPda,
            eventQ: eventQPda,
            authority: authority.publicKey, // Assuming this is the expected owner
          },
          signers: [authority],
        });
        console.log("Bid cancelled");
        let bids2 = await program.account.orders.fetch(bidsPda);
        console.log(bids2);
      }
    });
    
 
    })
});