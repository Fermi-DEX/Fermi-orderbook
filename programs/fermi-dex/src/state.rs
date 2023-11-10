use anchor_lang::{prelude::*, accounts::account_info};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, Approve},
};
//use solana_sdk::instruction::{AccountMeta, Instruction};

use anchor_spl::token::accessor::authority;
use enumflags2::{bitflags, BitFlags};
use resp;


use crate::utils2::*;
use crate::errors::ErrorCode;


#[account]
#[derive(Default)]
pub struct Market {
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,

    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,

    pub coin_lot_size: u64,
    pub pc_lot_size: u64,

    pub coin_deposits_total: u64,
    pub pc_deposits_total: u64,

    pub bids: Pubkey,
    pub asks: Pubkey,

    pub req_q: Pubkey,
    pub event_q: Pubkey,

    pub authority: Pubkey,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum RequestFlag {
    NewOrder = 0x01,
    CancelOrder = 0x02,
    Bid = 0x04,
    PostOnly = 0x08,
    ImmediateOrCancel = 0x10,
    DecrementTakeOnSelfTrade = 0x20,
}


#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct JitStruct {
    pub side: Side,
    pub maker: bool,
    pub native_qty_paid: u64,
    pub native_qty_received: u64,
    pub order_id: u128,
    pub owner: Pubkey,
    pub owner_slot: u8,
}

#[derive(Copy, Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct RequestQueueHeader {
    pub next_seq_num: u64,
}

impl RequestQueueHeader {
    pub const MAX_SIZE: usize = 8;
}

#[account]
#[derive(Default)]
pub struct RequestQueue {
    pub header: RequestQueueHeader,
}

impl RequestQueue {
    pub const MAX_SIZE: usize = RequestQueueHeader::MAX_SIZE;

    pub fn gen_order_id(&mut self, limit_price: u64, side: Side) -> u128 {
        let seq_num = self.gen_seq_num();
        let upper = (limit_price as u128) << 64;
        let lower = match side {
            Side::Bid => !seq_num,
            Side::Ask => seq_num,
        };
        upper | (lower as u128)
    }

    pub fn gen_seq_num(&mut self) -> u64 {
        let seq_num = self.header.next_seq_num;
        self.header.next_seq_num += 1;
        seq_num
    }
}


#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum EventFlag {
    Fill = 0x1,
    Out = 0x2,
    Bid = 0x4,
    Maker = 0x8,
    ReleaseFunds = 0x10,
    Finalise = 0x20,
}



pub enum EventView {
    Fill {
        side: Side,
        maker: bool,
        native_qty_paid: u64,
        native_qty_received: u64,
        order_id: u128,
        owner: Pubkey,
        owner_slot: u8,
        finalised: u8,
        cpty: Pubkey,
        order_id_second: u128,
    },
    Out {
        side: Side,
        release_funds: bool,
        native_qty_unlocked: u64,
        native_qty_still_locked: u64,
        order_id: u128,
        owner: Pubkey,
        owner_slot: u8,
        finalised: u8,
    },
    Finalise {
        side: Side,
        maker: bool,
        native_qty_paid: u64,
        native_qty_received: u64,
        order_id: u128,
        owner: Pubkey,
        owner_slot: u8,
        finalised: u8,
        cpty: Pubkey,
    },
}



pub struct EventQueueIterator<'a> {
    pub queue: &'a EventQueue,
    pub index: u64,
}

#[derive(Copy, Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct Order {
    pub order_id: u128,
    pub qty: u64,
    pub owner: Pubkey,
    pub owner_slot: u8,
}

#[repr(packed)]
#[zero_copy]
pub struct EventQueueHeader {
    pub head: u64,
    pub count: u64,
    pub seq_num: u64,
}

#[account(zero_copy)]
#[repr(packed)]
pub struct EventQueue {
    pub header: EventQueueHeader,
    pub head: u64,
    pub buf: [Event; 100], // Used zero_copy to expand eventsQ size
}

#[account]
#[derive(Default)]
pub struct Orders<const T: bool> {
    pub sorted: Vec<Order>,
}

pub type Bids = Orders<true>;
pub type Asks = Orders<false>;

pub struct RequestProceeds {
    pub coin_unlocked: u64,
    pub native_pc_unlocked: u64,

    pub coin_credit: u64,
    pub native_pc_credit: u64,

    pub coin_debit: u64,
    pub native_pc_debit: u64,
    pub jit_data: Vec<JitStruct>,
}

pub enum RequestView {
    NewOrder {
        side: Side,
        order_type: OrderType,
        order_id: u128,
        max_coin_qty: u64,
        native_pc_qty_locked: Option<u64>,
        owner_slot: u8,
        owner: Pubkey,
    },
    CancelOrder {
        side: Side,
        order_id: u128,
        cancel_id: u64,
        expected_owner_slot: u8,
        expected_owner: Pubkey,
    },

}

pub struct OrderBook<'a> {
    pub bids: &'a mut Bids,
    pub asks: &'a mut Asks,
    pub market: &'a Market,
}

pub struct NewOrderParams {
    pub side: Side,
    pub order_type: OrderType,
    pub order_id: u128,
    pub max_coin_qty: u64,
    pub native_pc_qty_locked: Option<u64>,
    pub owner: Pubkey,
    pub owner_slot: u8,
}

pub struct OrderRemaining {
    pub coin_qty_remaining: u64,
    pub native_pc_qty_remaining: Option<u64>,
}


#[repr(packed)]
#[zero_copy]
pub struct Event {
    pub event_flags: u8,
    pub owner_slot: u8,

    pub native_qty_released: u64,
    pub native_qty_paid: u64,

    pub order_id: u128,
    pub owner: Pubkey,
    pub finalised: u8,
    pub order_id_second: u128,
    // pub cpty: Pubkey, // Uncomment this if you want it to be public
}


// User owner value to track cpty

impl EventQueueHeader {
    pub const MAX_SIZE: usize = 8 + 8 + 8;

    pub fn head(&self) -> u64 {
        self.head
    }
    pub fn set_head(&mut self, value: u64) {
        self.head = value;
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn set_count(&mut self, value: u64) {
        self.count = value;
    }
    pub fn incr_event_id(&mut self) {
        self.seq_num += 1;
    }
    pub fn decr_event_id(&mut self, n: u64) {
        self.seq_num -= n;
    }
}




impl EventQueue {
    pub const MAX_SIZE: usize = EventQueueHeader::MAX_SIZE + 20 * Event::MAX_SIZE;

    #[inline]
    pub fn len(&self) -> u64 {
        return self.head;
    } 
       

    #[inline]
    pub fn full(&self) -> bool {
        self.header.count() as usize == self.buf.len()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.header.count() == 0
    }

    #[inline]
    pub fn push_back(&mut self, value: Event) -> Result<()> {
        if self.full() {
            let _ = self.pop_front();
        }

        //let slot = Some(peek_front_mut());
        let _ = self.pop_front();

        let slot = ((self.header.head() + self.header.count()) as usize) % self.buf.len();
        self.buf[slot] = value;

        let count = self.header.count();
        //self.header.set_count(count + 1);

        //self.header.incr_event_id();

        Ok(())
    }

    #[inline]
    pub fn peek_front(&self) -> Option<&Event> {
        if self.empty() {
            return None;
        }
        Some(&self.buf[self.header.head() as usize])
    }

    #[inline]
    pub fn peek_front_mut(&mut self) -> Option<&mut Event> {
        if self.empty() {
            return None;
        }
        Some(&mut self.buf[self.header.head() as usize])
    }

    #[inline]
    pub fn pop_front(&mut self) -> Result<Event> {
        require!(!self.empty(), ErrorCode::EmptyQueue);

        let value = self.buf[self.header.head() as usize];

        let count = self.header.count();
        self.header.set_count(count - 1);

        let head = self.header.head();
        self.header.set_head((head + 1) % self.buf.len() as u64);

        Ok(value)
    }


   
    }




impl Order {
    pub const MAX_SIZE: usize = 16;

    pub fn price_from_order_id(order_id: u128) -> u64 {
        (order_id >> 64) as u64
    }

    pub fn price(&self) -> u64 {
        Order::price_from_order_id(self.order_id)
    }
}


impl<const T: bool> Orders<T> {
    pub const MAX_SIZE: usize = 8 + 4 + 32 * Order::MAX_SIZE;

    pub fn find_bbo(&self) -> Result<&Order> {
        require!(self.sorted.len() > 0, ErrorCode::EmptyOrders);
        Ok(&self.sorted[0])
    }

    pub fn find_bbo_mut(&mut self) -> Result<&mut Order> {
        require!(self.sorted.len() > 0, ErrorCode::EmptyOrders);
        Ok(&mut self.sorted[0])
    }

    pub fn insert(&mut self, order: Order) -> Result<()> {
        self.sorted.push(order.clone());
        let mut is_found = false;
        for i in 0..(self.sorted.len() - 1) {
            if T {
                if self.sorted[i].price() < order.price() {
                    self.sorted[i] = order;
                    is_found = true;
                }
            } else {
                if self.sorted[i].price() > order.price() {
                    self.sorted[i] = order;
                    is_found = true;
                }
            }
            if is_found {
                self.sorted[i + 1] = self.sorted[i];
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, order_id: u128) -> Result<()> {
        let mut is_found = false;
        for i in 0..(self.sorted.len() - 1) {
            if self.sorted[i].order_id == order_id {
                is_found = true;
            }
            if is_found {
                self.sorted[i] = self.sorted[i + 1];
            }
        }
        self.sorted.pop();

        Ok(())
    }

    pub fn delete_worst(&mut self) -> Result<Order> {
        require!(!self.sorted.is_empty(), ErrorCode::EmptyOrders);
        Ok(self.sorted.pop().unwrap())
    }
}



macro_rules! impl_incr_method {
    ($method:ident, $var:ident) => {
        #[allow(unused)]
        pub fn $method(&mut self, $var: u64) {
            self.$var = self.$var.checked_add($var).unwrap();
        }
    };
}

impl RequestProceeds {
    impl_incr_method!(unlock_coin, coin_unlocked);
    impl_incr_method!(unlock_native_pc, native_pc_unlocked);
    impl_incr_method!(credit_coin, coin_credit);
    impl_incr_method!(credit_native_pc, native_pc_credit);
    impl_incr_method!(debit_coin, coin_debit);
    impl_incr_method!(debit_native_pc, native_pc_debit);
}


impl<'a> OrderBook<'a> {
    pub fn new_order(
        &mut self,
        params: NewOrderParams,
        event_q: &mut EventQueue,
        proceeds: &mut RequestProceeds,
    ) -> Result<Option<OrderRemaining>> {
        let NewOrderParams {
            side,
            order_type,
            order_id,
            owner,
            owner_slot,
            mut max_coin_qty,
            mut native_pc_qty_locked,
        } = params;
        let (mut post_only, mut post_allowed) = match order_type {
            OrderType::Limit => (false, true),
            OrderType::ImmediateOrCancel => (false, false),
            OrderType::PostOnly => (true, true),
        };
        //check Order impls for sourcing payer acc.
        let limit_price = Order::price_from_order_id(order_id);
        let mut limit = 10;
        loop {
            if limit == 0 {
                // Stop matching and release funds if we're out of cycles
                post_only = true;
                post_allowed = true;
            }

            let remaining_order = match side {
                Side::Bid => {
                //let deposit_vault = pc_vault;
                self.new_bid(
                    NewBidParams {
                        max_coin_qty,
                        native_pc_qty_locked: native_pc_qty_locked.unwrap(),
                        limit_price: Some(limit_price),
                        order_id,
                        owner,
                        owner_slot,
                        post_only,
                        post_allowed,
                    },
                    event_q,
                    proceeds,
                )},
                Side::Ask => {
                    
                    self.new_ask(
                        NewAskParams {
                            max_qty: max_coin_qty,
                            limit_price,
                            order_id,
                            owner,
                            owner_slot,
                            post_only,
                            post_allowed,
                        },
                        event_q,
                        proceeds,
                    )
                }
            }?;
            if limit == 0 {
                return Ok(remaining_order);
            }
            limit -= 1;
            match remaining_order {
                Some(remaining_order) => {
                    max_coin_qty = remaining_order.coin_qty_remaining;
                    native_pc_qty_locked = remaining_order.native_pc_qty_remaining;
                }
                None => return Ok(None),
                
            };
        }
    }
}

pub struct NewBidParams {
    pub max_coin_qty: u64,
    pub native_pc_qty_locked: u64,
    pub limit_price: Option<u64>,
    pub order_id: u128,
    pub owner: Pubkey,
    pub owner_slot: u8,
    pub post_only: bool,
    pub post_allowed: bool,
}

pub struct NewAskParams {
    pub max_qty: u64,
    pub limit_price: u64,
    pub order_id: u128,
    pub owner: Pubkey,
    pub owner_slot: u8,
    pub post_only: bool,
    pub post_allowed: bool,
}

pub struct CancelOrderParams {
    pub side: Side,
    pub order_id: u128,
    pub expected_owner: Pubkey,
    pub expected_owner_slot: u8,
}
    


#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Market::MAX_SIZE,
        seeds = [b"market".as_ref(), coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = pc_mint,
        associated_token::authority = market,
    )]
    pub pc_vault: Box<Account<'info, TokenAccount>>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = 8 + Bids::MAX_SIZE,
        seeds = [b"bids".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub bids: Box<Account<'info, Bids>>,
    #[account(
        init,
        payer = authority,
        space = 8 + Asks::MAX_SIZE,
        seeds = [b"asks".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub asks: Box<Account<'info, Asks>>,

    #[account(
        init,
        payer = authority,
        space = 8 + RequestQueue::MAX_SIZE,
        seeds = [b"req-q".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub req_q: Box<Account<'info, RequestQueue>>,
    #[account(
        //zero,
        init,
        payer = authority,
        space = 8 * 1264,
        seeds = [b"event-q".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub event_q: AccountLoader<'info, EventQueue>,

    //pub event_q: Box<Account<'info, EventQueue>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Copy, Debug, Clone, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum OrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
}

#[account]
#[derive(Default)]
pub struct OpenOrders {
    pub is_initialized: bool,

    pub market: Pubkey,
    pub authority: Pubkey,

    pub native_coin_free: u64,
    pub native_pc_free: u64,

    pub native_coin_total: u64,
    pub native_pc_total: u64,

    pub free_slot_bits: u8,
    pub is_bid_bits: u8,
    pub orders: [u128; 8],
}


#[derive(Accounts)]
//#[instruction(side: Side)]

pub struct FinaliseMatch<'info>{
    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub open_orders_owner: Box<Account<'info, OpenOrders>>,

    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub open_orders_cpty: Box<Account<'info, OpenOrders>>,

    #[account(
        seeds = [b"market".as_ref(), coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = pc_mint,
        associated_token::authority = market,
    )]
    pub pc_vault: Account<'info, TokenAccount>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(mut)]
    pub bids: Box<Account<'info, Bids>>,
    #[account(mut)]
    pub asks: Box<Account<'info, Asks>>,

    #[account(mut)]
    pub req_q: Box<Account<'info, RequestQueue>>,
    #[account(mut)]
    pub event_q: AccountLoader<'info, EventQueue>,

    #[account(
        mut,
        //constraint = market.check_payer_mint(payer.mint, side) @ ErrorCode::WrongPayerMint,
        token::authority = authority,
    )]
    pub pcpayer: Account<'info, TokenAccount>,

    #[account(
        mut,
        //constraint = market.check_payer_mint(payer.mint, side) @ ErrorCode::WrongPayerMint,
        token::authority = authority,
    )]
    pub coinpayer: Account<'info, TokenAccount>,
    //pub event_q: Box<Account<'info, EventQueue>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    //#[account(mut)]
    //pub authority_cpty: Account<'info, AccountInfo>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,



}


#[derive(Accounts)]
#[instruction(side: Side)]
pub struct NewOrder<'info> {
    #[account(
        init_if_needed,
        space = 8 + OpenOrders::MAX_SIZE,
        payer = authority,
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub open_orders: Box<Account<'info, OpenOrders>>,

    #[account(
        seeds = [b"market".as_ref(), coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = pc_mint,
        associated_token::authority = market,
    )]
    pub pc_vault: Account<'info, TokenAccount>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = market.check_payer_mint(payer.mint, side) @ ErrorCode::WrongPayerMint,
        token::authority = authority,
    )]
    pub payer: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bids: Box<Account<'info, Bids>>,
    #[account(mut)]
    pub asks: Box<Account<'info, Asks>>,

    #[account(mut)]
    pub req_q: Box<Account<'info, RequestQueue>>,
    #[account(mut)]
    //pub event_q: Box<Account<'info, EventQueue>>,
    pub event_q: AccountLoader<'info, EventQueue>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
//#[instruction(side: Side)]

pub struct NewMatch<'info>{
    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub open_orders_owner: Box<Account<'info, OpenOrders>>,

    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority_second.key().as_ref()],
        bump,
    )]
    //#[account(mut)]
    pub open_orders_counterparty: Box<Account<'info, OpenOrders>>,


   #[account(
      seeds = [b"market".as_ref(), coin_mint.key().as_ref(), pc_mint.key().as_ref()],
      bump,
    )] 
   // #[account(mut)]
    pub market: Box<Account<'info, Market>>,
    /*
    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>,
*/
    /*#[account(
        mut,
        associated_token::mint = pc_mint,
        associated_token::authority = market,
    )] */
    #[account(mut)]
    pub pc_vault: Account<'info, TokenAccount>, 

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,
    /*
    #[account(mut)]
    pub bids: Box<Account<'info, Bids>>,
    #[account(mut)]
    pub asks: Box<Account<'info, Asks>>, */

    #[account(mut)]
    pub req_q: Box<Account<'info, RequestQueue>>,
    #[account(mut)]
    pub event_q: AccountLoader<'info, EventQueue>,
    pub authority: Signer<'info>,


    #[account(mut)]
    pub authority_second: Signer<'info>,

    #[account(
        mut,
       // constraint = market.check_payer_mint(payer.mint, side) @ ErrorCode::WrongPayerMint,
        //token::authority = authority_second,
    )]
    pub pcpayer: Account<'info, TokenAccount>,
 

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,



}

#[derive(Accounts)]
pub struct NewMatchAsk<'info>{
    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub open_orders_owner: Box<Account<'info, OpenOrders>>,

    #[account(
        seeds = [b"open-orders".as_ref(), market.key().as_ref(), authority_second.key().as_ref()],
        bump,
    )]
   // #[account(mut)]
    pub open_orders_counterparty: Box<Account<'info, OpenOrders>>,


    #[account(
        seeds = [b"market".as_ref(), coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    /*
    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>,
*/
    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>, 

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,
    /*
    #[account(mut)]
    pub bids: Box<Account<'info, Bids>>,
    #[account(mut)]
    pub asks: Box<Account<'info, Asks>>, */

    #[account(mut)]
    pub req_q: Box<Account<'info, RequestQueue>>,
    #[account(mut)]
    pub event_q: AccountLoader<'info, EventQueue>,
    pub authority: Signer<'info>,


    #[account(mut)]
    pub authority_second: Signer<'info>,

    #[account(
        mut,
       // constraint = market.check_payer_mint(payer.mint, side) @ ErrorCode::WrongPayerMint,
       // token::authority = authority_second,
    )]
    pub coinpayer: Account<'info, TokenAccount>,
 

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,



}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub market: Box<Account<'info, Market>>,
    #[account(mut)]
    pub payer: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub open_orders: Box<Account<'info, OpenOrders>>,
    //pub system_program: Program<'info, System>,
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub open_orders: Box<Account<'info, OpenOrders>>,

    #[account(mut)]
    pub market: Box<Account<'info, Market>>,
    #[account(mut)]
    pub bids: Box<Account<'info, Bids>>,
    #[account(mut)]
    pub asks: Box<Account<'info, Asks>>,
    #[account(mut)]
    pub event_q: AccountLoader<'info, EventQueue>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub market: Box<Account<'info, Market>>,
    #[account(mut)]
    pub payer: Account<'info, TokenAccount>,
    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = coin_mint,
        associated_token::authority = market,
    )]
    pub coin_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = pc_mint,
        associated_token::authority = market,
    )]
    pub pc_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub open_orders: Box<Account<'info, OpenOrders>>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

}