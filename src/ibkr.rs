use reqwest::blocking::{Client, Response};
use std::{collections::HashMap, error::Error, process::exit};

use crate::structs::AccountResponse;

pub(crate) struct IBKR {
    discount_value: Option<f64>,
    domain: Option<String>,
    port: Option<String>,
    client: Option<Client>,
    account_id: Option<String>,
    spx_id: Option<String>,
    conids_map: Option<HashMap<String, HashMap<String, HashMap<f64, String>>>>,
}

impl IBKR {
    pub(crate) fn new() -> Self {
        IBKR {
            discount_value: None,
            domain: None,
            port: None,
            client: None,
            account_id: None,
            spx_id: None,
            conids_map: None,
        }
    }

    pub(crate) fn init(
        &mut self,
        discount_value: f64,
        domain: String,
        port: String,
        dates_slice: Vec<String>,
        strike_slice: HashMap<String, HashMap<String, Vec<f64>>>,
    ) -> Result<(), Box<dyn Error>> {
        self.discount_value = Some(discount_value);
        self.domain = Some(domain);
        self.port = Some(port);
        match self.get_account_id() {
            Ok(account_id) => {
                self.account_id = Some(account_id);
            }
            Err(e) => {
                eprintln!("Failed to get account ID: {}", e);
                exit(1)
            }
        }
        //self.spx_id = Some(self.get_spx_conid());
        //self.conids_map = Some(self.get_conids_map(&dates_slice, &strike_slice, &self.spx_id));
        Ok(())
    }

    // Function that sends a GET request for portfolio ID
    fn get_account_id(&self) -> Result<String, Box<dyn Error>> {
        let search_url: String = format!(
            "https://{}:{}/v1/api/portfolio/accounts",
            self.domain.as_ref().unwrap(),
            self.port.as_ref().unwrap()
        );

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(search_url)
            .header("Connection", "keep-alive")
            .send()?;

        if response.status().is_success() {
            let account_result: Vec<AccountResponse> = response.json()?;
            if let Some(first_account) = account_result.get(0) {
                Ok(first_account.id.clone())
            } else {
                eprintln!("No account found in the response");
                exit(1);
            }
        } else {
            eprintln!("Error: {}", response.status());
            exit(1);
        }
    }
}
