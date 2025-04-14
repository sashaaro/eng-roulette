use crate::domain::repository::Tx2pcID;
use anyhow::bail;
use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct InternalBillingService {
    pub(crate) client: reqwest::Client,
}

#[derive(Deserialize, Serialize)]
struct Income {
    user_id: i64,
    amount: i64,
    tx_id: Option<Tx2pcID>,
}
#[async_trait]
impl crate::domain::service::BillingService for InternalBillingService {
    // async fn income(&self, user_id: i64, amount: i64) -> anyhow::Result<()> {
    //     let resp = self.client.post("http://localhost:8000/income").json(&Income {
    //         user_id,
    //         amount,
    //         tx_id: None,
    //     }).send()
    //         .await?
    //         .json::<String>()
    //         .await?;
    //     println!("{:#?}", resp);
    //     Ok(())
    // }

    // async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i64, amount: i64) -> anyhow::Result<()> {
    //     let resp = self.client.post("http://localhost:8081/prepare_expense").json(&Income {
    //         user_id,
    //         amount,
    //         tx_id: Some(tx_id),
    //     }).send()
    //         .await?
    //         .text()
    //         .await?;
    //     println!("{:#?}", resp);
    //     Ok(())
    // }

    async fn commit_expense(&self, tx_id: Tx2pcID) -> anyhow::Result<()> {
        #[derive(Deserialize, Serialize)]
        struct Req {
            tx_id: Tx2pcID,
        }

        match self
            .client
            .post("http://localhost:8081/commit_expense")
            .json(&Req { tx_id: tx_id })
            .send()
            .await?
            .status()
        {
            StatusCode::OK => Ok(()),
            _ => bail!("failed to commit expense"),
        }
    }
}
