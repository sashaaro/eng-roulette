use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use crate::domain::repository::Tx2pcID;

#[derive(Clone)]
pub struct InternalBillingService {
    pub(crate) client: reqwest::Client
}

#[derive(Deserialize, Serialize)]
struct income {
    user_id: i64,
    amount: i64,
    tx_id: Option<Tx2pcID>
}
#[async_trait]
impl crate::domain::service::BillingService for InternalBillingService {
    async fn income(&self, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
        let resp = self.client.post("http://localhost:8000/income").json(&income{
            user_id,
            amount,
            tx_id: None,
        }).send()
            .await?
            .json::<String>()
            .await?;
        println!("{:#?}", resp);
        Ok(())
    }

    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
        let resp = self.client.post("http://localhost:8081/prepare_expense").json(&income{
            user_id,
            amount,
            tx_id: Some(tx_id),
        }).send()
            .await?
            .text()
            .await?;
        println!("{:#?}", resp);
        Ok(())
    }

    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        #[derive(Deserialize, Serialize)]
        struct req {
            tx_id: Tx2pcID
        }

        match self.client.post("http://localhost:8081/commit_expense").json(&req{
            tx_id: tx_id,
        }).send().await?.status() {
            StatusCode::OK => Ok(()),
            _ => Err(Box::new(MyError::new("wrong"))),
        }
    }
}



#[derive(Debug)]
struct MyError {
    details: String
}

impl MyError {
    fn new(msg: &str) -> MyError {
        MyError{details: msg.to_string()}
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for MyError {

}