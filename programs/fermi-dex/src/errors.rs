use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCodeCustom {
    #[msg("Wrong payer mint")]
    WrongPayerMint,
    #[msg("Wrong market")]
    WrongMarket,
    #[msg("Wrong authority")]
    WrongAuthority,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Transfer failed")]
    TransferFailed,

    #[msg("Already initialized")]
    AlreadyInitialized,

    #[msg("Queue already full")]
    QueueAlreadyFull,
    #[msg("Empty queue")]
    EmptyQueue,

    #[msg("Too many open orders")]
    TooManyOpenOrders,

    #[msg("Slot is not free")]
    SlotIsNotFree,

    #[msg("Empty orders")]
    EmptyOrders,
    #[msg("Orders already full")]
    OrdersAlreadyFull,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Insufficient native qty locked")]
    InvalidLocked,

    #[msg("OrderNotFound")]
    OrderNotFound,

    #[msg("InvalidAuthority")]
    InvalidAuthority,

    #[msg("BothEventsAlreadyFinalised")]
    BothEventsFinalised,

    #[msg("ThisSideAlreadyFinalised")]
    SideAlreadyFinalised,

    #[msg("EventNotExpired")]
    FinalizeNotExpired,

    #[msg("EventAlreadyFinalised")]
    EventFinalised,

    #[msg("WrongSideProvided")]
    WrongSide,

    #[msg("Error")]
    Error,

    #[msg("ApprovalFailed")]
    ApprovalFailed,
}
