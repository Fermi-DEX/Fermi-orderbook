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
import * as BufferLayout from 'buffer-layout';
            import { PublicKey, Connection } from '@solana/web3.js';
            import BN from 'bn.js';
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
  } from "./utils/contx.ts";

const {Keypair} = require("@solana/web3.js");
const secretKey = JSON.parse(fs.readFileSync("/Users/dm/.config/solana/id.json"));
const secretKeynew = JSON.parse(fs.readFileSync("/Users/dm/Documents/fermi_labs/basic/keypair2/keypair2.json"));

//kp3 = Bob (ask)
//kp4 = Alice (bid)
const secretKeySecond = JSON.parse(fs.readFileSync("./kp3/key.json"));
const secretKeyThird = JSON.parse(fs.readFileSync("./kp4/key.json"));


const keypair2 = Keypair.fromSecretKey(new Uint8Array(secretKeySecond));
const keypair = Keypair.fromSecretKey(new Uint8Array(secretKeyThird));

//const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
//const keypair = anchor.web3.Keypair.generate();

const authority = keypair;
const authority_second = keypair2;

let openOrdersPda: anchor.web3.PublicKey;
let openOrdersPdaBump: number;
let openOrders_secondPda: anchor.web3.PublicKey;
let openOrders_secondPdaBump: number;

async function fetchTokenBalance(mintAddress: string, userAddress: string) {
    const connection = new Connection("http://127.0.0.1:8899");
    
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
describe('#finalize-order', async () => {
    it('Finalize order - sell @ 19 successful', async () => {
        console.log('testing new finalise')
      {
        const provider = anchor.AnchorProvider.env();
/*
        const rpcUrl = 'https://api.devnet.solana.com';  // You can replace this with the appropriate RPC URL for your network.
        const rpcUrlLocal = 'http://localhost:8899';
        const wallet = new anchor.Wallet(authority_second);
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
            authority_second.publicKey,
            false,
          );
        const authorityCoinTokenAccount = await spl.getAssociatedTokenAddress(
            new anchor.web3.PublicKey(coinMint),
            authority.publicKey,
            false,
          );
        
            console.log("openorders");

        [openOrdersPda, openOrdersPdaBump] =
          await anchor.web3.PublicKey.findProgramAddress(
            [
              Buffer.from('open-orders', 'utf-8'),
              new anchor.web3.PublicKey(marketPda).toBuffer(),
              authority.publicKey.toBuffer(),
            ],
            new anchor.web3.PublicKey(programId),
          );

          [openOrders_secondPda, openOrders_secondPdaBump] = 
          await anchor.web3.PublicKey.findProgramAddress(
          [
            Buffer.from('open-orders', 'utf-8'),
            new anchor.web3.PublicKey(marketPda).toBuffer(),
            authority_second.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey(programId),
            );
          
          const eventsQ2 = await program.account.eventQueue.fetch(eventQPda);
          //let i = -1;
          //console.log(eventsQ2['buf'][1]);
          let order_id;
          let event_slot;
          console.log(authority);
          /*
          for(let i=0; i<eventsQ2['buf'].length; i++){
            //i+=1;
            let event = eventsQ2['buf'][i];
            //console.log(event.flag);
            if (event.flags=="0x1"){
              const event_slot = i;
              const order_id = event.order_id;
            }
          } */
          let base_order_id = 498062089990157893629;
          let base_event_slot = 3;
          let base_event_slot2 = 5;
      
          console.log(base_order_id);
          console.log('test finalise match with event slot + order id');
          console.log(authorityCoinTokenAccount.toString());
          console.log(authorityPcTokenAccount.toString());

          const objectsToCheck = {
            openOrdersPda,
            openOrders_secondPda,
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

          const eventQ = await program.account.eventQueue.fetch(eventQPda);
       //console.log(eventQ);
        //finalize bid side

        
        console.log("finalizing bid");
          await program.methods
            .finaliseMatchesBid(
              base_event_slot,
              base_event_slot2,
              //new anchor.web3.PublicKey(pcVault),
              //new anchor.web3.PublicKey(coinVault),
              //authorityPcTokenAccount,
              //authorityCoinTokenAccount,
              //new anchor.BN(0),
              //authority.PublicKey,
            )
            .accounts({
              //programId,
              openOrdersOwner: openOrdersPda,
              openOrdersCounterparty: openOrders_secondPda,
              market: marketPda,
              //coinVault,
              pcVault: pcVault,
              coinMint: coinMint,
              pcMint: pcMint,
              //payer: authorityPcTokenAccount,
              //bids: bidsPda,
              //asks: asksPda,
              reqQ: reqQPda,
              eventQ: eventQPda,
              authority: authority.publicKey,
              authority_second: authority_second.publicKey,
              pcpayer: authorityPcTokenAccount,
              coinpayer: authorityCoinTokenAccount,
            })
            .signers([authority])
            .rpc();
            console.log("finalized side Bid!");
            console.log("finalizing ask!");

            //finalize ask side
            await program.methods
            .finaliseMatchesAsk(
              base_event_slot,
              base_event_slot2,
              //new anchor.web3.PublicKey(pcVault),
              //new anchor.web3.PublicKey(coinVault),
              //authorityPcTokenAccount,
              //authorityCoinTokenAccount,
              //new anchor.BN(0),
              //authority.PublicKey,
            )
            .accounts({
              //programId,
              openOrdersOwner: openOrdersPda,
              openOrdersCounterparty: openOrders_secondPda,
              market: marketPda,
              //coinVault,
              coinVault,
              coinMint: coinMint,
              pcMint: pcMint,
              //payer: authorityPcTokenAccount,
              //bids: bidsPda,
              //asks: asksPda,
              reqQ: reqQPda,
              eventQ: eventQPda,
              authority: authority.publicKey,
              //pcpayer: authorityCoinTokenAccount,
              coinpayer: authorityCoinTokenAccount,
              authority_second: authority_second.publicKey,
            })
            .signers([authority])
            .rpc();
            
            
            // Define the layout of the data
            const EventLayout = BufferLayout.struct([
              BufferLayout.u8('event_flags'),
              BufferLayout.u8('owner_slot'),
              BufferLayout.blob(8, 'native_qty_released'),
              BufferLayout.blob(8, 'native_qty_paid'),
              BufferLayout.blob(16, 'order_id'),
              BufferLayout.blob(32, 'owner'),
              BufferLayout.u8('finalised'),
            ]);
            
            const EventQueueLayout = BufferLayout.struct([
              BufferLayout.blob(8, 'head'),
              BufferLayout.blob(8, 'count'),
              BufferLayout.blob(8, 'seq_num'),
              BufferLayout.seq(EventLayout, 100, 'events'),
            ]);
            
            // Connection to the network
            const connection = new Connection('https://api.devnet.solana.com');
            
            // Public key of the account
            //const eventQPda = new PublicKey('ACCOUNT_PUBLIC_KEY_HERE');
            
            // Fetch account data
            connection.getAccountInfo(new PublicKey(eventQPda)).then(accountInfo => {
              if (accountInfo && accountInfo.data) {
                // Decode the data using the layout
                const decodedData = EventQueueLayout.decode(accountInfo.data);
            
                // Additional conversions if needed
                decodedData.head = new BN(decodedData.head.reverse()).toString();
                decodedData.count = new BN(decodedData.count.reverse()).toString();
                decodedData.seq_num = new BN(decodedData.seq_num.reverse()).toString();
                // Handle other fields similarly...
              decodedData.events.forEach(event => {
                  event.native_qty_released = bufferToNumber(event.native_qty_released);
                  event.native_qty_paid = bufferToNumber(event.native_qty_paid);
                  event.order_id = bufferToBigInt(event.order_id).toString();
                  event.order_id_second = bufferToBigInt(event.order_id_second).toString(); // Added conversion
                  event.owner = event.owner.toString('hex');
                });
            
                console.log(decodedData);
                console.log("decoded data");
              }
            });
            console.log("decoded data");

            
            // Helper functions to convert buffer to number and BigInt
            function bufferToNumber(buffer: Buffer): number {
              return new BN(Array.from(buffer).reverse()).toNumber();
            }
            
            function bufferToBigInt(buffer: Buffer): BigInt {
              return BigInt('0x' + Array.from(buffer).reverse().map(b => b.toString(16).padStart(2, '0')).join(''));
            }
        

        console.log('completed finalize order sell price: 22');
      
        const openOrders = await program.account.openOrders.fetch(
          openOrdersPda,
        );
        /*
        //console.log(openOrders);
        const bids = await program.account.orders.fetch(bidsPda);
        //console.log(bids);
        const asks = await program.account.orders.fetch(asksPda);
        //console.log(asks);*/
        //const eventQ = await program.account.eventQueue.fetch(eventQPda);
        console.log(eventQ);
        /*
        const pcbal = await fetchTokenBalance(pcMint, authorityPcTokenAccount.toString());
        const coinbal = await fetchTokenBalance(coinMint, authorityCoinTokenAccount.toString());
        console.log("Ask placed at price: 25 successful");
        console.log("PC token balance: {}", pcbal);  ;
        console.log("Coin token balance: {}", coinbal);  ; */
      }
    })
    })
});