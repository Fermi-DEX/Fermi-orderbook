use anchor_lang::{prelude::*, accounts::account_info};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, Approve},
};
//use solana_sdk::instruction::{AccountMeta, Instruction};

use anchor_spl::token::accessor::authority;
use enumflags2::{bitflags, BitFlags};
use resp;
//extern crate bitflags;

mod utils2;
mod state;
mod errors;

use utils2::*;
use state::*;
use errors::*;
use crate::errors::ErrorCode;


declare_id!("4jde1a6MyoiwLVqB6UH5mBJp3gbpk1wcth8TZJfnf1V9");
//local
//declare_id!("HViUPBVkNo9v9y24N7qForgibiGGT3vQgbHjJnaScBMW");

#[program]
pub mod fermi_dex {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        coin_lot_size: u64,
        pc_lot_size: u64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        //let market = &mut ctx.accounts.market;
        market.coin_vault = ctx.accounts.coin_vault.key();
        market.pc_vault = ctx.accounts.pc_vault.key();
        market.coin_mint = ctx.accounts.coin_mint.key();
        market.pc_mint = ctx.accounts.pc_mint.key();
        market.coin_lot_size = coin_lot_size;
        market.pc_lot_size = pc_lot_size;
        market.coin_deposits_total = 0;
        market.pc_deposits_total = 0;
        market.bids = ctx.accounts.bids.key();
        market.asks = ctx.accounts.asks.key();
        market.req_q = ctx.accounts.req_q.key();
        market.event_q = ctx.accounts.event_q.key();
        market.authority = ctx.accounts.authority.key();

        Ok(())
    }

    pub fn cancel_bid(
        ctx: Context<CancelOrder>,
        order_id: u128,
        expected_owner: Pubkey,
    ) -> Result<()> {
        let bids = &mut ctx.accounts.bids;
        let event_q = &mut ctx.accounts.event_q.load_mut();
        let openorders = &mut ctx.accounts.open_orders;
        let authority = ctx.accounts.authority.key();

        //check openorders owner
         
        require!(openorders.authority == authority, ErrorCode::OrderNotFound);

        //check the order is owned by this user
        let mut x = 0;
        let mut slot: usize = 0;
        for (i, order) in openorders.orders.iter().enumerate() {
            let mut order_int = *order;
            if order_int == order_id {
                x = 1;
                slot = i;
            }
        }
        require!(x == 1, ErrorCode::OrderNotFound); 

        //remove order from orderbook
        let mut order_book = OrderBook {
            bids,
            asks: &mut ctx.accounts.asks,
            market: &mut ctx.accounts.market,
        };

        order_book.cancel_order_bid(true, order_id, expected_owner)?;

        //remove order from openorders
        //let res, err = openorders.remove_order(slot.try_into().unwrap());
        openorders.remove_order(slot.try_into().map_err(|_| ErrorCode::OrderNotFound)?)?;


        msg!("cancelled bid: {}", order_id);
        Ok(())
    }


        
    

    pub fn cancel_ask(
        ctx: Context<CancelOrder>,
        order_id: u128,
        expected_owner: Pubkey,
    ) -> Result<()> {
        let asks = &mut ctx.accounts.asks;
        let event_q = &mut ctx.accounts.event_q.load_mut();
        let openorders = &mut ctx.accounts.open_orders;
        let authority = ctx.accounts.authority.key();


        //check openorders owner
        require!(openorders.authority == authority, ErrorCode::OrderNotFound);

        //check the order is owned by this user
        let mut x = 0;
        let mut slot: usize = 0;
        for (i, order) in openorders.orders.iter().enumerate() {
            let mut order_int = *order;
            if order_int == order_id {
                x = 1;
                slot = i;
            }
        }
        require!(x == 1, ErrorCode::OrderNotFound); 

        let mut order_book = OrderBook {
            bids: &mut ctx.accounts.bids,
            asks,
            market: &mut ctx.accounts.market,
        };

        order_book.cancel_order_ask(false, order_id, expected_owner)?;


        //remove order from openOrders
        openorders.remove_order(slot.try_into().map_err(|_| ErrorCode::OrderNotFound)?)?;

        msg!("cancelled ask: {}", order_id);


        Ok(())
    }

    pub fn deposit_pc_tokens(
        ctx: Context<DepositTokens>,
        amount: u64,
    ) -> Result<()> {
        // Construct the transfer instruction
        msg!("Starting deposit_tokens function");
  
        let token_program = &ctx.accounts.token_program;

        let transfer_ix = Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_ix); 
        msg!("Transferred tokens!");
        
        // Execute the transfer
        anchor_spl::token::transfer(cpi_ctx, amount).map_err(|err| match err {
            _ => error!(ErrorCode::TransferFailed),
        })?; 
        
        
        ctx.accounts.open_orders.native_pc_free = ctx.accounts
            .open_orders
            .native_pc_free
            .checked_add(amount)
            .ok_or(ErrorCode::Error)?;
        
        Ok(())
    }

    pub fn deposit_coin_tokens(
        ctx: Context<DepositTokens>,
        amount: u64,
    ) -> Result<()> {
        // Construct the transfer instruction
        let token_program = &ctx.accounts.token_program;

        let transfer_ix = Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_ix);
        msg!("Transferred tokens!");
        
        // Execute the transfer
        anchor_spl::token::transfer(cpi_ctx, amount).map_err(|err| match err {
            _ => error!(ErrorCode::TransferFailed),
        })?;
        
        // Credit the balance to openOrders
        ctx.accounts.open_orders.native_coin_free = ctx.accounts
            .open_orders
            .native_coin_free
            .checked_add(amount)
            .ok_or(ErrorCode::Error)?;
        
        Ok(())
    }

     pub fn withdraw_coins(
        ctx: Context<WithdrawTokens>,
        amount: u64,
    ) -> Result<()> {
        let program_id = ctx.program_id;
        let open_orders = &mut ctx.accounts.open_orders;
        let market = &mut ctx.accounts.market;
        let coin_vault = &ctx.accounts.coin_vault;
        let pc_vault = &ctx.accounts.pc_vault;
        let payer = &ctx.accounts.payer;
        
        let authority = &ctx.accounts.authority;
        let token_program = &ctx.accounts.token_program;
        let coin_mint = &ctx.accounts.coin_mint;
        let pc_mint = &ctx.accounts.pc_mint;
        let (market_pda, bump_seed) = Pubkey::find_program_address(
            &[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
            &program_id
        );

        //Validation: owner of openorders is the authority
        require!(open_orders.authority == authority.key(), ErrorCode::InvalidAuthority);

       //Validation of the user's openorders balance
        require!(open_orders.native_coin_free >= amount, ErrorCode::InsufficientFunds);


       // Signing the transaction with the market PDA and bump seed.
        let market_seed = b"market";
        
        let coin_mint_key = coin_mint.key();
        let pc_mint_key = pc_mint.key();

        let coin_mint_seed = coin_mint_key.as_ref();
        let pc_mint_seed = pc_mint_key.as_ref();

        let bump_seed_arr: &[u8] = &[bump_seed];

        let seed_slices: [&[u8]; 4] = [market_seed, coin_mint_seed, pc_mint_seed, bump_seed_arr];
        let seeds: &[&[&[u8]]] = &[&seed_slices];


        let transfer_ix = Transfer {
            from: coin_vault.to_account_info(),
            to: payer.to_account_info(),
            authority: market.to_account_info(),  // Using the market PDA as the authority.
        };
    
        // Construct the context with the market PDA and bump seed.
        let cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            transfer_ix,
            seeds,
            //&[&[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref(), &[seeds]]]
        );
    
        anchor_spl::token::transfer(cpi_ctx, amount).map_err(|err| match err {
            _ => error!(ErrorCode::TransferFailed),
        })?;
        msg!("tokens withdrawn");

        // Reduce balance from user's OpenOrders account
        open_orders.native_coin_free = open_orders.native_coin_free.checked_sub(amount).ok_or(ErrorCode::Error)?;

        Ok(())
    }

    pub fn withdraw_tokens(
        ctx: Context<WithdrawTokens>,
        amount: u64,
    ) -> Result<()> {
        let program_id = ctx.program_id;
        let open_orders = &mut ctx.accounts.open_orders;
        let market = &mut ctx.accounts.market;
        let coin_vault = &ctx.accounts.coin_vault;
        let pc_vault = &ctx.accounts.pc_vault;
        let payer = &ctx.accounts.payer;
        
        let authority = &ctx.accounts.authority;
        let token_program = &ctx.accounts.token_program;
        let coin_mint = &ctx.accounts.coin_mint;
        let pc_mint = &ctx.accounts.pc_mint;
        let (market_pda, bump_seed) = Pubkey::find_program_address(
            &[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
            &program_id
        );

        // Validation: owner of openorders is the authority
        require!(open_orders.authority == authority.key(), ErrorCode::InvalidAuthority);

        // Validation of the user's openorders balance
        require!(open_orders.native_pc_free >= amount, ErrorCode::InsufficientFunds);
       

       // Signing the transaction with the market PDA and bump seed.
        let market_seed = b"market";
        
        let coin_mint_key = coin_mint.key();
        let pc_mint_key = pc_mint.key();

        let coin_mint_seed = coin_mint_key.as_ref();
        let pc_mint_seed = pc_mint_key.as_ref();

        let bump_seed_arr: &[u8] = &[bump_seed];

        let seed_slices: [&[u8]; 4] = [market_seed, coin_mint_seed, pc_mint_seed, bump_seed_arr];
        let seeds: &[&[&[u8]]] = &[&seed_slices];

        let transfer_ix = Transfer {
            from: pc_vault.to_account_info(),
            to: payer.to_account_info(),
            authority: market.to_account_info(),  // Using the market PDA as the authority.
        };
    
        // Construct the context with the market PDA and bump seed.
        let cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            transfer_ix,
            seeds,
            //&[&[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref(), &[seeds]]]
        );
    
        anchor_spl::token::transfer(cpi_ctx, amount).map_err(|err| match err {
            _ => error!(ErrorCode::TransferFailed),
        })?;
        msg!("tokens withdrawn");

        // Reduce balance from user's OpenOrders account
        open_orders.native_pc_free = open_orders.native_pc_free.checked_sub(amount).ok_or(ErrorCode::Error)?;

        Ok(())
    }





    pub fn new_order(
        ctx: Context<NewOrder>,
        side: Side,
        limit_price: u64,
        max_coin_qty: u64,
        max_native_pc_qty: u64,
        order_type: OrderType,
    ) -> Result<()> {
        let open_orders = &mut ctx.accounts.open_orders;
        let market = &mut ctx.accounts.market;
        let coin_vault = &ctx.accounts.coin_vault;
        let pc_vault = &ctx.accounts.pc_vault;
        let payer = &ctx.accounts.payer;
        let bids = &mut ctx.accounts.bids;
        let asks = &mut ctx.accounts.asks;
        let req_q = &mut ctx.accounts.req_q;
        let event_q = &mut ctx.accounts.event_q.load_mut();
        let authority = &ctx.accounts.authority;
        let token_program = &ctx.accounts.token_program;
        let coin_mint = &ctx.accounts.coin_mint;
        let pc_mint = &ctx.accounts.pc_mint;


        if !open_orders.is_initialized {
            open_orders.init(market.key(), authority.key())?;
        } else {
            require!(
                open_orders.market.key() == market.key(),
                ErrorCode::WrongMarket
            );
            require!(
                open_orders.authority.key() == authority.key(),
                ErrorCode::WrongAuthority
            );
        }
        
        let deposit_amount;
        let deposit_vault;
        let cpty_vault;
        let native_pc_qty_locked;
        match side {
            Side::Bid => {
                let lock_qty_native = max_native_pc_qty;
                native_pc_qty_locked = Some(lock_qty_native);
                let free_qty_to_lock = lock_qty_native.min(open_orders.native_pc_free);
                let total_deposit_amount = lock_qty_native - free_qty_to_lock;
                //deposit_amount = total_deposit_amount * 2/100; //marginal deposit up front
                deposit_amount = total_deposit_amount; //for test with matching, L1044
                deposit_vault = pc_vault;
                cpty_vault = coin_vault;
                
                market.pc_deposits_total = market
                    .pc_deposits_total
                    .checked_add(deposit_amount)
                    .unwrap();
            }
            Side::Ask => {
                native_pc_qty_locked = None;
                let lock_qty_native = max_coin_qty
                    .checked_mul(market.coin_lot_size)
                    .ok_or(error!(ErrorCode::InsufficientFunds))?;
                let free_qty_to_lock = lock_qty_native.min(open_orders.native_coin_free);
                let total_deposit_amount = lock_qty_native - free_qty_to_lock;
                //deposit_amount = total_deposit_amount * 2/100; //marginal deposit up front
                deposit_amount = total_deposit_amount; //for test with matching, L1044
                deposit_vault = coin_vault;
                cpty_vault = pc_vault;
               
                market.coin_deposits_total = market
                    .coin_deposits_total
                    .checked_add(deposit_amount)
                    .unwrap();
            }
        }

        let order_id = req_q.gen_order_id(limit_price, side);
        if open_orders.free_slot_bits == 0 {
            open_orders.remove_order(0);
        }
        let owner_slot = open_orders.add_order(order_id, side)?;
        let request = RequestView::NewOrder {
            side,
            order_type,
            order_id,
            owner: open_orders.key(),
            owner_slot,
            max_coin_qty,
            native_pc_qty_locked,
        };
        let jitdata: Vec<JitStruct> = vec![];
        let mut proceeds = RequestProceeds {
            coin_unlocked: 0,
            native_pc_unlocked: 0,
            coin_credit: 0,
            native_pc_credit: 0,
            coin_debit: 0,
            native_pc_debit: 0,
            jit_data: jitdata,
        };
        let mut order_book = OrderBook { bids, asks, market };

        // matching occurs at this stage
        order_book.process_request(&request, &mut event_q.as_mut().unwrap(), &mut proceeds)?;
        //msg!(event_q[1].side);
        //let jit_data = vec![];

        {
            let coin_lot_size = market.coin_lot_size;

            let RequestProceeds {
                coin_unlocked,
                coin_credit,

                native_pc_unlocked,
                native_pc_credit,

                coin_debit,
                native_pc_debit,
                jit_data,
            } = proceeds;
            let native_coin_unlocked = coin_unlocked.checked_mul(coin_lot_size).unwrap();
            let native_coin_credit = coin_credit.checked_mul(coin_lot_size).unwrap();
            let native_coin_debit = coin_debit.checked_mul(coin_lot_size).unwrap();

          

            let others = jit_data;
            


            msg!("going to loop!");
            for p in others {
                msg!("heya {}", p.owner);
                let mut owner_slot = p.owner_slot;
                msg!("this is the way {}", owner_slot);
                let mut owner_order = open_orders.orders[usize::from(owner_slot)];
                //let qty = owner_order.qty;
                let mut deposits = p.native_qty_paid;
                //let mut owner_deposits = owner_order.deposits;
                msg!("owner qty {}", owner_order);
                msg!("dep {}", deposits);
}
               //_orders_mut.native_pc_free <= open_orders_mut.native_pc_total)?;
        }


        let matched_amount_pc = proceeds.native_pc_credit;
        let matched_amount_coin = proceeds.coin_credit;

        // if order is not crossed, creator is maker, and only needs to approve tokens.

        if deposit_amount > 0 {
            //if !crossed {
            msg!("approval amount {}", deposit_amount);
            //msg!("deposit vault {}", deposit_vault);
            //msg!("approval vault {}", payer);

            let transfer_ix = Approve {
                to: payer.to_account_info(),
                delegate: market.to_account_info(),
                authority: authority.to_account_info(), // authority.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_ix);
            //let marginal_deposit = cpi_ctx * 2 / 100
            anchor_spl::token::approve(cpi_ctx, deposit_amount).map_err(|err| match err {
                _ => error!(ErrorCode::TransferFailed),
            })?;
            msg!("tokens approved for later spending.");
        }
        
        
        msg!("matched amount {}", matched_amount_coin);
       


           Ok(())
       }


       
    pub fn finalise_matches_bid(
                ctx: Context<NewMatch>,
                event1_slot: u8,
                event2_slot: u8,
            ) -> Result<()> {
                let program_id = ctx.program_id;
                let open_orders_auth = &mut ctx.accounts.open_orders_owner;
                let open_orders_cpty = &mut ctx.accounts.open_orders_counterparty;
                let market = &ctx.accounts.market;
                let pc_vault = &ctx.accounts.pc_vault;
                let req_q = &mut ctx.accounts.req_q;
                let event_q = &mut ctx.accounts.event_q.load_mut()?;
                let authority = &ctx.accounts.authority;
                let token_program = &ctx.accounts.token_program;
                let coin_mint = &ctx.accounts.coin_mint;
                let pc_mint = &ctx.accounts.pc_mint;
                let payerpc = &ctx.accounts.pcpayer;
            
                let event1: Event = event_q.buf[usize::from(event1_slot)];
                let event2: Event = event_q.buf[usize::from(event2_slot)];
            
                let event1_orderid = event1.order_id;
                let event2_orderid = event2.order_id;
                let event1_orderidsecond = event1.order_id_second;
                let event2_orderidsecond = event2.order_id_second;
            
                msg!("event1 orderid is {}", event1_orderid);
                msg!("event1 orderidsecond is {}", event1_orderidsecond);
                msg!("event2 orderid is {}", event2_orderid);
                msg!("event2 orderidsecond is {}", event2_orderidsecond);
            
                require!(event1.order_id_second == event2.order_id, Error);
            
                let events: Vec<Event> = vec![event1, event2];
                let mut order_id_general: u128 = 0;
                let mut first_event_done: bool = false;
                let mut eventBidFinalised: bool = false;
                let mut eventAskFinalised: bool = false;
            
                
                for (index, parsed_event) in events.iter().enumerate() {
                   
                    let sider; // u8 for side
                    match BitFlags::<EventFlag>::from_bits(parsed_event.event_flags) {
                        Ok(flags) => {
                            let side = EventFlag::flags_to_side(flags);
                            msg!("The side derived from parsed_event.event_flags is: {:?}", side);
                        },
                        Err(_) => {
                            msg!("Error: Invalid flags detected: {:?}", parsed_event.event_flags);
                        }
                    }
                    
                    
                    let flags = BitFlags::<EventFlag>::from_bits(parsed_event.event_flags).unwrap_or(BitFlags::empty());

                    let side = EventFlag::flags_to_side(flags);
                    if side == Side::Bid {
                        sider = 1;
                    }
                    else {
                        sider = 2;
                    }
                    msg!("side is {}", sider);
                //match side {
                  //  Side::Bid => {
                //let sider = 1;
                
                    if sider == 1 {
                        let mut qty_pc = parsed_event.native_qty_paid;
                        let mut qty_coin = parsed_event.native_qty_released;
                        let mut available_funds = open_orders_auth.native_pc_free;
                        msg!("the available funds is {}", available_funds);
                        msg!("the required funds are {}", qty_pc);
            
                        //let mut deposit_amount = qty_pc / 1000;
                        let mut deposit_amount = qty_pc / (1000* market.pc_lot_size) ;
                        msg!("Deposit amt {}", deposit_amount);
                        let mut cpty_deposit_amt = qty_coin;
                        let mut deposit_vault = pc_vault;
            
                        if deposit_amount > 0 {
                            // Derive the market's PDA and bump seed.
                            let (market_pda, bump_seed) = Pubkey::find_program_address(
                                &[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
                                &program_id
                            );

                           
                            let market_seed = b"market";
                            
                            let coin_mint_key = coin_mint.key();
                            let pc_mint_key = pc_mint.key();

                            let coin_mint_seed = coin_mint_key.as_ref();
                            let pc_mint_seed = pc_mint_key.as_ref();

                            let bump_seed_arr: &[u8] = &[bump_seed];

                            let seed_slices: [&[u8]; 4] = [market_seed, coin_mint_seed, pc_mint_seed, bump_seed_arr];
                            let seeds: &[&[&[u8]]] = &[&seed_slices];
                           
                            let transfer_ix = Transfer {
                                from: payerpc.to_account_info(),
                                to: deposit_vault.to_account_info(),
                                authority: market.to_account_info(),  // Using the market PDA as the authority.
                            };
                        
                            // Construct the context with the market PDA and bump seed.
                            let cpi_ctx = CpiContext::new_with_signer(
                                token_program.to_account_info(),
                                transfer_ix,
                                seeds,
                                //&[&[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref(), &[seeds]]]
                            );
                        
                            anchor_spl::token::transfer(cpi_ctx, deposit_amount).map_err(|err| match err {
                                _ => error!(ErrorCode::TransferFailed),
                            })?;
                            let fin: u8 = 1;
                            let owner = parsed_event.owner;
                            open_orders_auth.credit_unlocked_pc(deposit_amount);
                            let bidder_fill = Event::new(EventView::Finalise {
                             side: Side::Ask,
                             maker: true,
                             native_qty_paid:  parsed_event.native_qty_paid,
                             native_qty_received: parsed_event.native_qty_released,
                             order_id: parsed_event.order_id,
                             owner: parsed_event.owner,
                             owner_slot: parsed_event.owner_slot,
                             finalised: fin,
                             cpty: owner,
                         });
                         //let idx = event_q.as_mut().unwrap().head + 1;
                         let mut event_slot = 1;
                         if index == 0 {
                            event_slot = event1_slot;
                         }
                         if index == 1 {
                            event_slot = event2_slot;
                         }
                         let idx = event_slot;
                         event_q.buf[idx as usize] = bidder_fill;
                         eventBidFinalised = true;
                        }
                        if cpty_deposit_amt > 0 {
                           //open_orders_cpty.credit_unlocked_coin(cpty_deposit_amt);
                        }
                        let mut remaining_funds = 0;
                        if remaining_funds > 0 {
                           
                            msg!("Newly locked PC for bidder {}", qty_pc);
                        }
                        if index == 0 {
                            open_orders_auth.native_pc_free  = open_orders_auth
                                .native_pc_free
                                .checked_add(qty_pc)
                                .unwrap();
                            // open_orders_auth.credit_unlocked_pc(deposit_amount);
                            }
                        if index == 1 {
                            open_orders_cpty.native_pc_free  = open_orders_cpty
                                .native_pc_free
                                .checked_add(deposit_amount)
                                .unwrap();
                            }
                    }
                    // Side::Ask => {
                    if sider == 2 {
                        let mut eventFin = parsed_event.finalised;
                        //let eventAskFinalised;
                        if eventFin == 1{
                            eventAskFinalised = true;
                        }
                        if eventFin == 0 {
                            eventAskFinalised == false;
                        }
                        
                    } 
                }
                //Settlement if both events are finalised
                
                if eventBidFinalised == true && eventAskFinalised == true {
                    //checked subtract pc from event1 owner
                   // open_orders_auth.native_pc_free -= event1.native_qty_paid;
                    open_orders_auth.native_pc_free = open_orders_auth
                                .native_pc_free
                                .checked_sub(event1.native_qty_paid)
                                .unwrap();
                            
                    //subtract coin from event2 owner
                    //open_orders_cpty.native_coin_free -= event2.native_qty_paid;
                    //checked sub
                    open_orders_cpty.native_coin_free = open_orders_cpty
                                .native_coin_free
                                .checked_sub(event2.native_qty_paid)
                                .unwrap();
                            
                    //add pc to event2 owner
                    open_orders_cpty.native_pc_free += event2.native_qty_released;
                    //add coin to event1 owner  
                    open_orders_auth.native_coin_free += event1.native_qty_released;

                    msg!("settlement completed!");

                }
            
                Ok(())
            }


            
        
            
            
            
            

            /// just in time transfers for ask side
pub fn finalise_matches_ask(
                ctx: Context<NewMatchAsk>,
                event1_slot: u8,
                event2_slot: u8,
            ) -> Result<()> {
                let program_id = ctx.program_id;
                let open_orders_auth = &mut ctx.accounts.open_orders_owner; //owner of event 1
                let open_orders_cpty = &mut ctx.accounts.open_orders_counterparty; // owner of event 2
                let market = &ctx.accounts.market;
                let coin_vault = &ctx.accounts.coin_vault;
                let req_q = &mut ctx.accounts.req_q;
                let event_q = &mut ctx.accounts.event_q.load_mut()?;
                let authority = &ctx.accounts.authority;
                let token_program = &ctx.accounts.token_program;
                let coin_mint = &ctx.accounts.coin_mint;
                let pc_mint = &ctx.accounts.pc_mint;
                let payercoin = &ctx.accounts.coinpayer;
            
                let event1: Event = event_q.buf[usize::from(event1_slot)];
                let event2: Event = event_q.buf[usize::from(event2_slot)];
            
                let event1_orderid = event1.order_id;
                let event2_orderid = event2.order_id;
                let event1_orderidsecond = event1.order_id_second;
                let event2_orderidsecond = event2.order_id_second;
            
                msg!("event1 orderid is {}", event1_orderid);
                msg!("event1 orderidsecond is {}", event1_orderidsecond);
                msg!("event2 orderid is {}", event2_orderid);
                msg!("event2 orderidsecond is {}", event2_orderidsecond);
            
                //require!(event1.order_id_second == event2.order_id, Error);
            
                let events: Vec<Event> = vec![event1, event2];
                let mut order_id_general: u128 = 0;
                let mut first_event_done: bool = false;
                let mut eventBidFinalised: bool = false;
                let mut eventAskFinalised: bool = false;
            
                //let parsed_event = events[1];
                //let mut sider = parsed_event.event_flags;
                for (index, parsed_event) in events.iter().enumerate() {
                    let sider;
                   
                    //let side = flags_to_side(parsed_event.event_flags);
                    let flags = BitFlags::<EventFlag>::from_bits(parsed_event.event_flags).unwrap_or(BitFlags::empty());

                    let side = EventFlag::flags_to_side(flags);
                    if side == Side::Bid {
                        sider = 1;
                    }
                    else {
                        sider = 2;
                    }
                    msg!("side is {}", sider);
                    //match side {
                        //Side::Ask => {
                       if sider == 2  {
                            //let mut qty_pc = parsed_event.native_qty_paid;
                            let mut qty_coin = parsed_event.native_qty_paid;
                            let mut available_funds = open_orders_auth.native_coin_free;
                            msg!("the available funds is {}", available_funds);
                            msg!("the required funds are {}", qty_coin);
                
                            //let mut deposit_amount = qty_pc / 1000;
                            let mut deposit_amount = qty_coin; //decimals already multiplied
                            msg!("Deposit amt {}", deposit_amount);
                            let mut cpty_deposit_amt = qty_coin;
                            let mut deposit_vault = coin_vault;
                
                            if deposit_amount > 0 {
                                // Derive the market's PDA and bump seed.
                                let (market_pda, bump_seed) = Pubkey::find_program_address(
                                    &[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
                                    &program_id
                                );

                            
                                let market_seed = b"market";
                                
                                let coin_mint_key = coin_mint.key();
                                let pc_mint_key = pc_mint.key();

                                let coin_mint_seed = coin_mint_key.as_ref();
                                let pc_mint_seed = pc_mint_key.as_ref();

                                let bump_seed_arr: &[u8] = &[bump_seed];

                                let seed_slices: [&[u8]; 4] = [market_seed, coin_mint_seed, pc_mint_seed, bump_seed_arr];
                                let seeds: &[&[&[u8]]] = &[&seed_slices];
                                
                                let transfer_ix = Transfer {
                                    from: payercoin.to_account_info(),
                                    to: deposit_vault.to_account_info(),
                                    authority: market.to_account_info(),  // Using the market PDA as the authority.
                                };
                            
                                // Construct the context with the market PDA and bump seed.
                                let cpi_ctx = CpiContext::new_with_signer(
                                    token_program.to_account_info(),
                                    transfer_ix,
                                    seeds,
                                    //&[&[b"market", coin_mint.key().as_ref(), pc_mint.key().as_ref(), &[seeds]]]
                                );
                            
                                anchor_spl::token::transfer(cpi_ctx, deposit_amount).map_err(|err| match err {
                                    _ => error!(ErrorCode::TransferFailed),
                                })?;
                                let fin: u8 = 1;
                                let owner = parsed_event.owner;
                                let asker_fill = Event::new(EventView::Finalise {
                                    side: Side::Ask,
                                    maker: true,
                                    native_qty_paid:  parsed_event.native_qty_paid,
                                    native_qty_received: parsed_event.native_qty_released,
                                    order_id: parsed_event.order_id,
                                    owner: parsed_event.owner,
                                    owner_slot: parsed_event.owner_slot,
                                    finalised: fin,
                                    cpty: owner,
                                });
                                //let idx = event_q.as_mut().unwrap().head + 1;
                                let mut event_slot = 0;
                                if index == 0 {
                                   event_slot = event1_slot;
                                }
                                if index == 1 {
                                   event_slot = event2_slot;
                                }
                                let idx = event_slot;
                                event_q.buf[idx as usize] = asker_fill;
                                let event_bid_finalised = true;
                            
                            //accounting
                            if index == 0 {
                            open_orders_auth.native_coin_free = open_orders_auth
                                .native_coin_free
                                .checked_add(deposit_amount)
                                .unwrap();
                            }
                            if index == 1 {
                                open_orders_cpty.native_coin_free = open_orders_cpty
                                    .native_coin_free
                                    .checked_add(deposit_amount)
                                    .unwrap();

                        }


                        /*
                        if cpty_deposit_amt > 0 {
                            open_orders_cpty.credit_unlocked_coin(cpty_deposit_amt);
                        } */
                        let mut remaining_funds = 1;
                        eventAskFinalised = true;
                        
                    }
                    //Side::Bid => {
                        if sider == 1 {
                            // check if event is finalised
                            let mut eventFin = parsed_event.finalised;
                            //let eventBidFinalised;
                            if eventFin == 1{
                                eventBidFinalised = true;
                            }
                            if eventFin == 0 {
                                eventBidFinalised == false;
                            }

                      
                    }
                //}
                }
            }
            //Settle funds
            if eventBidFinalised == true && eventAskFinalised == true {
                //checked subtract pc from event1 owner
                //open_orders_auth.native_pc_free -= event1.native_qty_paid;
                open_orders_auth.native_pc_free = open_orders_auth
                                .native_pc_free
                                .checked_sub(event1.native_qty_paid)
                                .unwrap();
                            
                //subtract coin from event2 owner
                //open_orders_cpty.native_coin_free -= event2.native_qty_paid;
                open_orders_cpty.native_coin_free = open_orders_cpty
                                .native_coin_free
                                .checked_sub(event2.native_qty_paid)
                                .unwrap();
                //add pc to event2 owner
                open_orders_cpty.native_pc_free += event2.native_qty_released;
                //add coin to event1 owner
                open_orders_auth.native_coin_free += event1.native_qty_released;
            
            }
                Ok(())
            
        }

            
         }
        