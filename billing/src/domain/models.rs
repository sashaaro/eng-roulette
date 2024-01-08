use sqlx::Encode;

#[derive(Encode)]
#[repr(i32)]
pub enum TxType {
    Income = 1,
    Expense = 0
}

pub struct TransactionHistory {
    pub id: i32,
    pub user_id: i32,
    pub tx_type: TxType,
    pub amount: i32
}

