use std::error::Error;
use serde::Deserialize;
use crate::domain::repository::Tx2pcID;

#[derive(Clone)]
pub struct InternalBillingService {
    pub(crate) client: reqwest::Client
}

#[derive(Deserialize)]
struct income {
    user_id: i32,
    amount: i32,
    tx_id: Option<Tx2pcID>
}

impl crate::domain::service::BillingService for InternalBillingService {
    async fn income(&self, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>> {
        let resp = self.client.post("http://localhost:8000/income").json(&income{
            user_id,
            amount,
        })
            .await?
            .json::<String>()
            .await?;
        println!("{:#?}", resp);
        Ok(())
    }

    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>> {
        let resp = self.client.post("http://localhost:8000/prepare_expense").json(&income{
            user_id,
            amount,
            tx_id: Some(tx_id),
        })
            .await?
            .json::<String>()
            .await?;
        println!("{:#?}", resp);
        Ok(())
    }

    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        #[derive(Deserialize)]
        struct req {
            tx_id: Tx2pcID
        }

        let resp = self.client.post("http://localhost:8000/commit_expense").json(&req{
            tx_id: tx_id,
        })
            .await?
            .json::<String>()
            .await?;
        println!("{:#?}", resp);
        Ok(())
    }
}