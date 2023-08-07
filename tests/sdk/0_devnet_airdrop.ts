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
  } from "./utils/consts_market4";

  import {createAssociatedTokenAccount, mintTo} from "./utils/utils"

const {Keypair} = require("@solana/web3.js");
const secretKey = JSON.parse(fs.readFileSync("/Users/dm/.config/solana/id.json"));
const secretKeynew = JSON.parse(fs.readFileSync("/Users/dm/Documents/fermi_labs/basic/keypair2/keypair2.json"));

const secretKeySecond = JSON.parse(fs.readFileSync("./kp3/key.json"));
const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));

//HARDCODE YOUR DEVNET PUBKEY HERE TO RECIEVE AIRDROPS
//'EN31BH6XonqZdwZrMpqtgHcQ8supSZqVhBEE5GhmVrN6'
//'HubyrMHSh2s5KXeTYRFhYbY32hVPrG8bbAre2AzewqRR'
const userpubkey = new anchor.web3.PublicKey('HubyrMHSh2s5KXeTYRFhYbY32hVPrG8bbAre2AzewqRR');
const keypair2 = Keypair.fromSecretKey(new Uint8Array(secretKeySecond));

let authorityCoinTokenAccount: anchor.web3.PublicKey;
//const authority = keypair2;
const authority = userpubkey;
//const authority2 = keypair;
let authorityPcTokenAccount: anchor.web3.PublicKey;
//let createAssociatedTokenAccount : anchor.web3.PublicKey;
//let minto : anchor.web3.PublicKey;
//let mintTo : anchor.web3.PublicKey
console.log("It's Bob's turn to get airdrops")
describe('fermi-dex-setup', () => {
  before(async () => {
    // Add your before hook here.
  });
describe('create ATA and airdrop', async () => {
  const provider = anchor.AnchorProvider.env();
  it('creating ata for usdc and bonk and airdropping on devnet', async () => {
    // test code here
  
    authorityCoinTokenAccount = await spl.getAssociatedTokenAddress(
      new anchor.web3.PublicKey(coinMint),
      authority,
      false,
    );

    authorityPcTokenAccount = await spl.getAssociatedTokenAddress(
      new anchor.web3.PublicKey(pcMint),
      authority,
      false,
    );

    console.log("dervei ATA done")

 // comment out if ATA is already created.
 
 
    await createAssociatedTokenAccount(
      provider,
      new anchor.web3.PublicKey(coinMint),
      authorityCoinTokenAccount,
      authority,
    );
    /*
    await createAssociatedTokenAccount(
      provider,
      new anchor.web3.PublicKey(pcMint),
      authorityPcTokenAccount,
      authority,
    );  
 */
    console.log("create ATA done")

    await mintTo(
      provider,
      new anchor.web3.PublicKey(coinMint),
      authorityCoinTokenAccount,
      BigInt('10000000000'),


    );

    console.log("mint coin done")

    await mintTo(
      provider,
      new anchor.web3.PublicKey(pcMint),
      authorityPcTokenAccount,
      BigInt('1000000000'),
    );
    console.log("sent to");
    console.log(authorityPcTokenAccount.toString());

    });
  })});