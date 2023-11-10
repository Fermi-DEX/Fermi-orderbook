use anchor_lang::{prelude::*, accounts::account_info};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, Approve},
};
//use solana_sdk::instruction::{AccountMeta, Instruction};

use anchor_spl::token::accessor::authority;
use enumflags2::{bitflags, BitFlags};
use resp;



use crate::state::*;
use crate::errors::*;

use crate::errors::ErrorCode;

impl Market {
    pub const MAX_SIZE: usize = 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32 + 32;

    #[inline]
    pub fn check_payer_mint(&self, payer_mint: Pubkey, side: Side) -> bool {
        match side {
            Side::Bid => {
                if payer_mint == self.pc_mint {
                    return true;
                }
                return false;
            }
            Side::Ask => {
                if payer_mint == self.coin_mint {
                    return true;
                }
                return false;
            }
        }
    }
}

impl EventView {
    pub fn side(&self) -> Side {
        match self {
            &EventView::Fill { side, .. } | &EventView::Out { side, .. } |  &EventView::Finalise { side, .. } => side,
        }
    }
}

impl EventFlag {
    #[inline]
    pub fn from_side(side: Side) -> BitFlags<Self> {
        match side {
            Side::Bid => EventFlag::Bid.into(),
            Side::Ask => BitFlags::empty(),
        }
    }

    #[inline]
    pub fn flags_to_side(flags: BitFlags<Self>) -> Side {
        if flags.contains(EventFlag::Bid) {
            Side::Bid
        } else {
            Side::Ask
        }
    }
}

impl Event {
    pub const MAX_SIZE: usize = 1 + 1 + 8 + 8 + 16 + 32 + 1 + 32;

    #[inline(always)]
    pub fn new(view: EventView) -> Self {
        match view {
            EventView::Fill {
                side,
                maker,
                native_qty_paid,
                native_qty_received,
                order_id,
                owner,
                owner_slot,
                finalised,
                cpty,
                order_id_second,
            } => {
                let mut flags = EventFlag::from_side(side) | EventFlag::Fill;
                if maker {
                    flags |= EventFlag::Maker;
                }
                let mut finalised: u8 = 0;
                Event {
                    event_flags: flags.bits(),
                    owner_slot,
                    native_qty_released: native_qty_received,
                    native_qty_paid,
                    order_id,
                    owner,
                    finalised,
                    order_id_second,
                    //cpty,
                }
            },

            EventView::Out {
                side,
                release_funds,
                native_qty_unlocked,
                native_qty_still_locked,
                order_id,
                owner,
                owner_slot,
                finalised,
            } => {
                let mut flags = EventFlag::from_side(side) | EventFlag::Out;
                if release_funds {
                    flags |= EventFlag::ReleaseFunds;
                }
                let mut finalised: u8 = 0;
                let mut cpty: Pubkey = owner;
                Event {
                    event_flags: flags.bits(),
                    owner_slot,
                    //finalised: finalised,
                    native_qty_released: native_qty_unlocked,
                    native_qty_paid: native_qty_still_locked,
                    order_id,
                    owner,
                    finalised,
                    order_id_second: 0,
                    //cpty
                }

            },

            EventView::Finalise {
                side,
                maker,
                native_qty_paid,
                native_qty_received,
                order_id,
                owner,
                owner_slot,
                finalised,
                cpty,
            } => {
                let mut flags = EventFlag::from_side(side) | EventFlag::Fill;
                if maker {
                    flags |= EventFlag::Maker;
                }
                //let mut finalsed= true;
                Event {
                    event_flags: flags.bits(),
                    owner_slot,
                    //finalised: finalised,
                    native_qty_released: native_qty_received,
                    native_qty_paid,
                    order_id,
                    owner,
                    finalised,
                    order_id_second:0,
                    //cpty,
                }
        }
    }
}

    
}

impl<'a> OrderBook<'a> {
    pub fn find_bbo(&self, side: Side) -> Result<&Order> {
        match side {
            Side::Bid => self.bids.find_bbo(),
            Side::Ask => self.asks.find_bbo(),
        }
    }

    pub fn find_bbo_mut(&mut self, side: Side) -> Result<&mut Order> {
        match side {
            Side::Bid => self.bids.find_bbo_mut(),
            Side::Ask => self.asks.find_bbo_mut(),
        }
    }

    pub fn process_request(
        &mut self,
        request: &RequestView,
        event_q: &mut EventQueue,
        proceeds: &mut RequestProceeds,
    ) -> Result<Option<RequestView>> {
        Ok(match *request {
            RequestView::NewOrder {
                side,
                order_type,
                order_id,
                max_coin_qty,
                native_pc_qty_locked,
                owner_slot,
                owner,
            } => self
                .new_order(
                    NewOrderParams {
                        side,
                        order_type,
                        order_id,
                        max_coin_qty,
                        native_pc_qty_locked,
                        owner_slot,
                        owner,
                    },
                    event_q,
                    proceeds,
                )?
                .map(|remaining| RequestView::NewOrder {
                    side,
                    order_type,
                    order_id,
                    max_coin_qty: remaining.coin_qty_remaining,
                    native_pc_qty_locked: remaining.native_pc_qty_remaining,
                    owner_slot,
                    owner,
                }),
            RequestView::CancelOrder {
                side,
                order_id,
                cancel_id: _,
                expected_owner,
                expected_owner_slot,
            } => {
                self.cancel_order(
                    CancelOrderParams {
                        side,
                        order_id,
                        expected_owner,
                        expected_owner_slot,
                    },
                    event_q,
                )?;
                None
            }
            /*


            RequestView::JitStruct { .. } => {
                msg!("jit it!");
                None
            }
            */
        })
    }
}

impl<'a> OrderBook<'a> {
    pub fn new_bid(
        &mut self,
        params: NewBidParams,
        event_q: &mut EventQueue,
        to_release: &mut RequestProceeds,
    ) -> Result<Option<OrderRemaining>> {
        let NewBidParams {
            max_coin_qty,
            native_pc_qty_locked,
            limit_price,
            order_id,
            owner,
            owner_slot,
            post_only,
            post_allowed,
        } = params;
        if post_allowed {
            require!(limit_price.is_some(), ErrorCode::InvalidPrice);
        }

        let coin_lot_size = self.market.coin_lot_size;
        let pc_lot_size = self.market.pc_lot_size;

        msg!("[OrderBook.new_bid] coin_lot_size: {}", coin_lot_size);
        msg!("[OrderBook.new_bid] pc_lot_size: {}", pc_lot_size);

        let max_pc_qty = native_pc_qty_locked / pc_lot_size;

        msg!("[OrderBook.new_bid] max_coin_qty: {}", max_coin_qty);
        msg!("[OrderBook.new_bid] native_pc_qty_locked: {}", native_pc_qty_locked);
        msg!("[OrderBook.new_bid] limit_price: {}", limit_price.unwrap());
        msg!("[OrderBook.new_bid] order_id: {}", order_id);
        msg!("[OrderBook.new_bid] post_only: {}", post_only);
        msg!("[OrderBook.new_bid] post_allowed: {}", post_allowed);

        let mut coin_qty_remaining = max_coin_qty;
        let mut pc_qty_remaining = max_pc_qty;
        let mut jit_data = vec![];
        
        msg!("bid inserted");
        let insert_result = self.bids.insert(Order {
            order_id,
            qty: max_coin_qty,
            owner,
            owner_slot,
        });
        if let Err(err) = insert_result {
            if err == error!(ErrorCode::OrdersAlreadyFull) {
                // boot out the least aggressive bid
                msg!("bids full! booting...");
                let order = self.bids.delete_worst()?;
                let out = Event::new(EventView::Out {
                    side: Side::Bid,
                    release_funds: true,
                    native_qty_unlocked: order.qty * order.price() * pc_lot_size,
                    native_qty_still_locked: 0,
                    order_id: order.order_id,
                    owner: order.owner,
                    owner_slot: order.owner_slot,
                    finalised: 0,
                });
                let idx = event_q.head + 1;
                msg!("event id is {}", idx);

                event_q.buf[idx as usize] = out;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Bid");
                msg!("event.release_funds: {}", "true");
                msg!("event.native_qty_unlocked: {}", order.qty * order.price() * pc_lot_size);
                msg!("event.order_id: {}", order.order_id);
                msg!("event.order_id_second: {}", 0);
                msg!("event.order: {}", order.owner);
                msg!("event.owner_slot: {}", order.owner_slot);
                msg!("event.finalised: {}", "0");
/*
                event_q
                    .push_back(out)
                    .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?; */
                self.bids.insert(Order {
                    order_id,
                    qty: max_coin_qty,
                    owner,
                    owner_slot,
                })?;
            }
}
        let crossed;
        let done = loop {
            let best_offer = match self.find_bbo_mut(Side::Ask) {
                Err(_) => {
                    crossed = false;
                    break true;
                }
                Ok(o) => o,
            };

            let trade_price = best_offer.price();
            crossed = limit_price
                .map(|limit_price| limit_price >= trade_price)
                .unwrap_or(true);
            // testing

            if !crossed || post_only {
                msg!("not crossed!");
                break true;
            }
            msg!("crossed!");
            let offer_size = best_offer.qty;
            let trade_qty = offer_size
                .min(coin_qty_remaining)
                .min(pc_qty_remaining / trade_price);

            if trade_qty == 0 {
                break true;
            }

            let native_maker_pc_qty = trade_qty * trade_price * pc_lot_size;
            
            let idx = event_q.head + 1;
            let maker_fill = Event::new(EventView::Fill {
                side: Side::Ask,
                maker: true,
                native_qty_paid: trade_qty * coin_lot_size,
                native_qty_received: native_maker_pc_qty,
                order_id: best_offer.order_id,
                owner: best_offer.owner,
                owner_slot: best_offer.owner_slot,
                finalised: 0,
                cpty: owner,
                order_id_second: order_id,
            });
            //let lenevents = event_q.len();
            //let idx = lenevents +1;
            
            //write maker side event to eventQ
            event_q.buf[idx as usize] = maker_fill;
            event_q.head +=1;
             //   .push_back(maker_fill)
             //   .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Ask");
                msg!("event.maker: {}", "true");
                msg!("event.native_qty_paid: {}", trade_qty * coin_lot_size);
                msg!("event.native_qty_received: {}", native_maker_pc_qty);
                msg!("event.order_id: {}", best_offer.order_id);
                msg!("event.order_id_second: {}", order_id);
                msg!("event.owner: {}", best_offer.owner);
                msg!("owner_slot: {}", best_offer.owner_slot);
                msg!("event.finalised: {}", "0");
                msg!("event.cpty_orderid: {}", order_id);



            best_offer.qty -= trade_qty;
            coin_qty_remaining -= trade_qty;
            pc_qty_remaining -= trade_qty * trade_price;

            //if order is filled, delete (ask) order.
            if best_offer.qty == 0 {
                let best_offer_id = best_offer.order_id;

                let event_out = Event::new(EventView::Out {
                    side: Side::Ask,
                    release_funds: true,
                    native_qty_unlocked: 0,
                    native_qty_still_locked: 0,
                    order_id: best_offer_id,
                    owner: best_offer.owner,
                    owner_slot: best_offer.owner_slot,
                    finalised: 0,
                });
                let idx = event_q.head + 1;
                msg!("event id is {}", idx);
                event_q.buf[idx as usize] = event_out;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Ask");
                msg!("event.release_funds: {}", "true");
                msg!("event.native_qty_unlocked: {}", "0");
                msg!("event.native_qty_still_locked: {}", "0");
                msg!("event.order_id: {}", best_offer_id);
                msg!("event.order_id_second: {}", 0);
                msg!("event.owner: {}", best_offer.owner);
                msg!("event.owner_slot: {}", best_offer.owner_slot);
                msg!("event.finalised: {}", "0");



            }

            break false;
        };

        msg!("[OrderBook.new_bid] crossed: {}", crossed);
        msg!("[OrderBook.new_bid] done: {}", done);
        msg!("[OrderBook.new_bid] countrerparty: {}", done);
        msg!("[OrderBook.new_bid] coin_qty_remaining: {}", coin_qty_remaining);
        msg!("[OrderBook.new_bid] pc_qty_remaining: {}", pc_qty_remaining);

        let native_accum_fill_price = (max_pc_qty - pc_qty_remaining) * pc_lot_size;
        let native_pc_qty_remaining = native_pc_qty_locked - native_accum_fill_price;

        msg!("[OrderBook.new_bid] native_accum_fill_price: {}", native_accum_fill_price);
        msg!("[OrderBook.new_bid] native_pc_qty_remaining: {}", native_pc_qty_remaining);

        {
            let coin_lots_received = max_coin_qty - coin_qty_remaining;
            let native_pc_paid = native_accum_fill_price;

            to_release.credit_coin(coin_lots_received);
            to_release.debit_native_pc(native_pc_paid);
            to_release.jit_data = jit_data;

            // multiple possible counterparties
            //if native_accum_fill_price > 0 {
                let taker_fill = Event::new(EventView::Fill {
                    side: Side::Bid,
                    maker: false,
                    native_qty_paid: native_pc_paid,
                    native_qty_received: coin_lots_received * coin_lot_size,
                    order_id,
                    owner,
                    owner_slot,
                    finalised: 0,
                    cpty: owner,
                    order_id_second: 0,
                });
                let idx = event_q.head + 1;
                msg!("event id is {}", idx);

                event_q.buf[idx as usize] = taker_fill;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Bid");
                msg!("event.maker: {}", "false");
                msg!("event.native_qty_paid: {}", native_pc_paid);
                msg!("event.native_qty_received: {}", coin_lots_received * coin_lot_size);
                msg!("event.order_id: {}", order_id);
                msg!("event.order_id_second: {}", 0);
                msg!("event.owner: {}", owner);
                msg!("event.owner_slot: {}", owner_slot);
                msg!("event.finalised: {}", "0");




/*
                event_q
                    .push_back(taker_fill)
                    .map_err(|_| ErrorCode::QueueAlreadyFull)?; */
           // }
        }

        if !done {
            if coin_qty_remaining > 0 && native_pc_qty_remaining > 0 {
                return Ok(Some(OrderRemaining {
                    coin_qty_remaining,
                    native_pc_qty_remaining: Some(native_pc_qty_remaining),
                }));
            }
        }

        let (coin_qty_to_post, pc_qty_to_keep_locked) = match limit_price {
            Some(price) if post_allowed && !crossed => {
                let coin_qty_to_post =
                    coin_qty_remaining.min(native_pc_qty_remaining / pc_lot_size / price);
                (coin_qty_to_post, coin_qty_to_post * price)
            }
            _ => (0, 0),
        };

        msg!("[OrderBook.new_bid] coin_qty_to_post: {}", coin_qty_to_post);
        msg!("[OrderBook.new_bid] pc_qty_to_keep_locked: {}", pc_qty_to_keep_locked);

        let out = {
            let native_qty_still_locked = pc_qty_to_keep_locked * pc_lot_size;
            let native_qty_unlocked = native_pc_qty_remaining - native_qty_still_locked;
            to_release.unlock_native_pc(native_qty_unlocked);

            let outer = Event::new(EventView::Out {
                side: Side::Bid,
                release_funds: false,
                native_qty_unlocked,
                native_qty_still_locked,
                order_id,
                owner,
                owner_slot,
                finalised: 0,
            });
            let idx = event_q.head + 1;
        msg!("event id is {}", idx);
        event_q.buf[idx as usize] = outer;
        event_q.head +=1;
        };
        let idx = event_q.head;

        let native_qty_still_locked = pc_qty_to_keep_locked * pc_lot_size;
            let native_qty_unlocked = native_pc_qty_remaining - native_qty_still_locked;
            to_release.unlock_native_pc(native_qty_unlocked);

        msg!("event.idx: {}", idx);
        msg!("event.side: {}", "Ask");
        msg!("event.release_funds: {}", "false");
        msg!("event.native_qty_unlocked: {}", native_qty_unlocked);
        msg!("event.native_qty_still_locked: {}", native_qty_still_locked);
        msg!("event.order_id: {}", order_id);
        msg!("event.order_id_second: {}", order_id);

        msg!("event.owner: {}", owner);
        msg!("owner_slot: {}", owner_slot);
        msg!("event.finalised: {}", "0");



        Ok(None)
    }
}

impl<'a> OrderBook<'a> {
    pub fn new_ask(
        &mut self,
        params: NewAskParams,
        event_q: &mut EventQueue,
        to_release: &mut RequestProceeds,
    ) -> Result<Option<OrderRemaining>> {
        let NewAskParams {
            max_qty,
            limit_price,
            order_id,
            owner,
            owner_slot,
            post_only,
            post_allowed,
        } = params;
        let mut unfilled_qty = max_qty;
        let mut accum_fill_price = 0;

        let pc_lot_size = self.market.pc_lot_size;
        let coin_lot_size = self.market.coin_lot_size;
        let mut jit_data = vec![];

        //begin matching
        let crossed;

        let insert_result = self.asks.insert(Order {
            order_id,
            qty: unfilled_qty,
            owner,
            owner_slot,
        });
        if let Err(err) = insert_result {
            if err == error!(ErrorCode::OrdersAlreadyFull) {
                // boot out the least aggressive offer
                msg!("offers full! booting...");
                let order = self.asks.delete_worst()?;
                let out = Event::new(EventView::Out {
                    side: Side::Ask,
                    release_funds: true,
                    native_qty_unlocked: order.qty * coin_lot_size,
                    native_qty_still_locked: 0,
                    order_id: order.order_id,
                    owner: order.owner,
                    owner_slot: order.owner_slot,
                    finalised: 0,
                });
                let idx = event_q.head + 1;
                msg!("idx is {}", idx);
                event_q.buf[idx as usize] = out;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Ask");
                msg!("event.release_funds: {}", true);
                msg!("event.native_qty_unlocked: {}", order.qty * coin_lot_size);
                msg!("event.native_qty_still_locked: {}", "0");
                msg!("event.order_id: {}", order.order_id);
                msg!("event.owner: {}", order.owner);
                msg!("event.owner_slot: {}", order.owner_slot);
                msg!("event.finalised: {}", "0");

/*
                event_q
                    .push_back(out)
                    .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;*/
                self.asks.insert(Order {
                    order_id,
                    qty: unfilled_qty,
                    owner,
                    owner_slot,
                })?;
            }
        }

        let done = loop {
            let best_bid = match self.find_bbo_mut(Side::Bid) {
                Err(_) => {
                    crossed = false;
                    break true;
                }
                Ok(o) => o,
            };

            let trade_price = best_bid.price();
            crossed = limit_price <= trade_price;

            if !crossed || post_only {
                break true;
            }

            let bid_size = best_bid.qty;
            let trade_qty = bid_size.min(unfilled_qty);

            if trade_qty == 0 {
                break true;
            }

            let native_maker_pc_qty = trade_qty * trade_price * pc_lot_size;
            let jit_struct = JitStruct {
                side: Side::Bid,
                maker: true,
                native_qty_paid: native_maker_pc_qty,
                native_qty_received: trade_qty * coin_lot_size,
                order_id: best_bid.order_id,
                owner: best_bid.owner,
                owner_slot: best_bid.owner_slot,
            };
            jit_data.push(jit_struct);
            msg!("data pushed to jitstruct");

            let maker_fill = Event::new(EventView::Fill {
                side: Side::Bid,
                maker: true,
                native_qty_paid: native_maker_pc_qty,
                native_qty_received: trade_qty * coin_lot_size,
                order_id: best_bid.order_id,
                owner: best_bid.owner,
                owner_slot: best_bid.owner_slot,
                finalised: 0,
                cpty: owner,
                order_id_second: order_id,
            });
            let idx = event_q.head + 1;
            event_q.buf[idx as usize] = maker_fill;
            event_q.head +=1;
            msg!("event.idx: {}", idx);
            msg!("event.side: {}", "Ask");
            msg!("event.maker: {}", "true");
            msg!("event.native_qty_paid: {}", trade_qty * coin_lot_size);
            msg!("event.native_qty_received: {}", trade_qty * coin_lot_size);
            msg!("event.order_id: {}", best_bid.order_id);
            msg!("event.order_id_second: {}", order_id);
            msg!("event.owner: {}", best_bid.owner);
            msg!("event.owner_slot: {}", best_bid.owner_slot);
            msg!("event.finalised: {}", "0");
            msg!("event.cpty_orderid: {}", order_id);



/*
            event_q
                .push_back(maker_fill)
                .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;*/

            best_bid.qty -= trade_qty;
            unfilled_qty -= trade_qty;
            accum_fill_price += trade_qty * trade_price;

            if best_bid.qty == 0 {
                let best_bid_id = best_bid.order_id;
                let out = Event::new(EventView::Out {
                    side: Side::Bid,
                    release_funds: true,
                    native_qty_unlocked: 0,
                    native_qty_still_locked: 0,
                    order_id: best_bid_id,
                    owner: best_bid.owner,
                    owner_slot: best_bid.owner_slot,
                    finalised: 0,
                });
                let idx = event_q.head + 1;
                event_q.buf[idx as usize] = out;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Bid");
                msg!("event.release_funds: {}", "true");
                msg!("event.native_qty_unlocked: {}", "0");
                msg!("event.native_qty_locked: {}", "0");
                msg!("event.order_id: {}", order_id);
                msg!("event.order_id_second: {}", 0);
                msg!("event.owner: {}", best_bid.owner);
                msg!("event.owner_slot: {}", best_bid.owner_slot);
                msg!("event.finalised: {}", "0");


                /*event_q
                    .push_back(Event::new(EventView::Out {
                        side: Side::Bid,
                        release_funds: true,
                        native_qty_unlocked: 0,
                        native_qty_still_locked: 0,
                        order_id: best_bid_id,
                        owner: best_bid.owner,
                        owner_slot: best_bid.owner_slot,
                    }))
                    .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;*/
                //self.bids.delete(best_bid_id)?;
            }

            break false;
        };

        let native_taker_pc_qty = accum_fill_price * pc_lot_size;

        {
            let net_taker_pc_qty = native_taker_pc_qty;
            let coin_lots_traded = max_qty - unfilled_qty;

            to_release.credit_native_pc(net_taker_pc_qty);
            to_release.debit_coin(coin_lots_traded);
            to_release.jit_data = jit_data;
            if native_taker_pc_qty > 0 {
                let taker_fill = Event::new(EventView::Fill {
                    side: Side::Ask,
                    maker: false,
                    native_qty_paid: coin_lots_traded * coin_lot_size,
                    native_qty_received: net_taker_pc_qty,
                    order_id,
                    owner,
                    owner_slot,
                    finalised: 0,
                    cpty: owner,
                    order_id_second: 0,
                });
                let idx = event_q.head + 1;
                event_q.buf[idx as usize] = taker_fill;
                event_q.head +=1;

                msg!("event.idx: {}", idx);
                msg!("event.side: {}", "Ask");
                msg!("event.maker: {}", "false");
                msg!("event.native_qty_paid: {}", coin_lots_traded * coin_lot_size);
                msg!("event.native_qty_received: {}", net_taker_pc_qty);
                msg!("event.order_id: {}", order_id);
                msg!("event.order_id_second: {}", 0);
                msg!("event.owner: {}", owner);
                msg!("event.owner_slot: {}", owner_slot);
                msg!("event.finalised: {}", "0");



/*
                event_q
                    .push_back(taker_fill)
                    .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;*/
            }
        }

        if !done {
            if unfilled_qty > 0 {
                return Ok(Some(OrderRemaining {
                    coin_qty_remaining: unfilled_qty,
                    native_pc_qty_remaining: None,
                }));
            }
        }

        if post_allowed && !crossed && unfilled_qty > 0 {
       
        } else {
            to_release.unlock_coin(unfilled_qty);
            let out = Event::new(EventView::Out {
                side: Side::Ask,
                release_funds: false,
                native_qty_unlocked: unfilled_qty * coin_lot_size,
                native_qty_still_locked: 0,
                order_id,
                owner,
                owner_slot,
                finalised: 0,
            });
            let idx = event_q.head + 1;
            event_q.buf[idx as usize] = out;
            event_q.head +=1;

            msg!("event.idx: {}", idx);
            msg!("event.side: {}", "Ask");
            msg!("event.release_funds: {}", false);
            msg!("event.native_qty_unlocked: {}", unfilled_qty * coin_lot_size);
            msg!("event.native_qty_still_locked: {}", "0");
            msg!("event.order_id: {}", order_id);
            msg!("event.order_id_second: {}", 0);
            msg!("event.owner: {}", owner);
            msg!("event.owner.slot: {}", owner_slot);
            msg!("event.finalised: {}", "0");
/*
            event_q
                .push_back(out)
                .map_err(|_| error!(ErrorCode::QueueAlreadyFull))?;*/
        }

        Ok(None)
    }
}

impl<'a> OrderBook<'a> {
    pub fn cancel_order(&mut self, params: CancelOrderParams, event_q: &mut EventQueue) -> Result<()> { 
        let CancelOrderParams {
            side,
            order_id,
            expected_owner,
            expected_owner_slot,
        } = params; 
        Ok(())
    }

    pub fn cancel_order_bid(&mut self, side: bool, order_id: u128, owner: Pubkey) -> Result<()> {
       
        //  pub fn remove_order_by_id_and_owner(&mut self, side: bool, order_id: u128, owner: Pubkey) -> Result<(), ErrorCode> {
        //let orders = if side { &mut *self.bids } else { &mut *self.asks };
        let orders = &mut *self.bids;
        orders.delete(order_id);

        
            Ok(())
        }

        pub fn cancel_order_ask(&mut self, side: bool, order_id: u128, owner: Pubkey) -> Result<()> {
       
            //  pub fn remove_order_by_id_and_owner(&mut self, side: bool, order_id: u128, owner: Pubkey) -> Result<(), ErrorCode> {
            //let orders = if side { &mut *self.bids } else { &mut *self.asks };
            let orders = &mut *self.asks;
            orders.delete(order_id);
    
            //if let Some(leaf_node) = self.orders_mut(side).remove_by_key(order_id) {
              //  } else {
                //    self.orders_mut(side).insert_leaf(&leaf_node).unwrap();
              //  }
                Ok(())
            }
        

        
    }


    impl OpenOrders {
        pub const MAX_SIZE: usize = 1 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 8 * 16;
    
        pub fn init(&mut self, market: Pubkey, authority: Pubkey) -> Result<()> {
            require!(!self.is_initialized, ErrorCode::AlreadyInitialized);
    
            self.is_initialized = true;
            self.market = market;
            self.authority = authority;
            self.free_slot_bits = std::u8::MAX;
    
            Ok(())
        }
    
        pub fn credit_unlocked_coin(&mut self, native_coin_amount: u64) {
            self.native_coin_total = self
                .native_coin_total
                .checked_add(native_coin_amount)
                .unwrap();
            self.native_coin_free = self.native_coin_free.checked_add(native_coin_amount).unwrap();
        }
    
        pub fn credit_locked_coin(&mut self, native_coin_amount: u64) {
            self.native_coin_total = self
                .native_coin_total
                .checked_add(native_coin_amount)
                .unwrap();
        }
    
        pub fn credit_unlocked_pc(&mut self, native_pc_amount: u64) {
            self.native_pc_total = self.native_pc_total.checked_add(native_pc_amount).unwrap();
            self.native_pc_free = self.native_pc_free.checked_add(native_pc_amount).unwrap();
        }
    
        pub fn credit_locked_pc(&mut self, native_pc_amount: u64) {
            self.native_pc_total = self.native_pc_total.checked_add(native_pc_amount).unwrap();
        }
    
        pub fn lock_free_coin(&mut self, native_coin_amount: u64) {
            self.native_coin_free = self
                .native_coin_free
                .checked_sub(native_coin_amount)
                .unwrap();
        }
    
       pub fn lock_free_pc(&mut self, native_pc_amount: u64) {
            self.native_pc_free = self.native_pc_free.checked_sub(native_pc_amount).unwrap();
        }
    
        pub fn unlock_coin(&mut self, native_coin_amount: u64) {
            self.native_coin_free = self
                .native_coin_free
                .checked_add(native_coin_amount)
                .unwrap();
            assert!(self.native_coin_free <= self.native_coin_total);
        }
    
        pub fn unlock_pc(&mut self, native_pc_amount: u64) {
            self.native_pc_free = self.native_pc_free.checked_add(native_pc_amount).unwrap();
            assert!(self.native_pc_free <= self.native_pc_total);
        }
    
        pub fn slot_is_free(&self, slot: u8) -> bool {
            let slot_mask = 1u8 << slot;
            self.free_slot_bits & slot_mask != 0
        }
    
        
    
        pub fn slot_side(&self, slot: u8) -> Option<Side> {
            let slot_mask = 1u8 << slot;
            if self.free_slot_bits & slot_mask != 0 {
                None
            } else if self.is_bid_bits & slot_mask != 0 {
                Some(Side::Bid)
            } else {
                Some(Side::Ask)
            }
        }
    
        pub fn remove_order(&mut self, slot: u8) -> Result<()> {
            // check_assert!(slot < 128)?;
            // check_assert!(!self.slot_is_free(slot))?;
            //require!(self.slot_is_free(slot), ErrorCode::SlotIsNotFree);
    
            let slot_mask = 1u8 << slot;
            self.orders[slot as usize] = 0;
            self.free_slot_bits |= slot_mask;
            self.is_bid_bits &= !slot_mask;
    
            Ok(())
        }
    
        pub fn add_order(&mut self, id: u128, side: Side) -> Result<u8> {
            //remove oldest order if openorders is full
            if self.free_slot_bits == 0 {
                self.remove_order(0)?;
            } 
            //require!(self.free_slot_bits != 0, ErrorCode::TooManyOpenOrders);
            let slot = self.free_slot_bits.trailing_zeros() as u8;
            require!(self.slot_is_free(slot), ErrorCode::SlotIsNotFree);
            let slot_mask = 1u8 << slot;
            self.free_slot_bits &= !slot_mask;
            match side {
                Side::Bid => {
                    self.is_bid_bits |= slot_mask;
                }
                Side::Ask => {
                    self.is_bid_bits &= !slot_mask;
                }
            };
            self.orders[slot as usize] = id;
            Ok(slot as u8)
        }
    }
    