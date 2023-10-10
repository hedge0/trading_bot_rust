use ordered_float::OrderedFloat;
use reqwest::blocking::{Client, ClientBuilder, Response};
use std::{collections::HashMap, error::Error, process::exit, thread::sleep, time::Duration};

use crate::{
    helpers::convert_date,
    structs::{AccountResponse, SecDefInfoResponse, SecDefResponse},
};

pub(crate) struct IBKR {
    discount_value: Option<f64>,
    domain: Option<String>,
    port: Option<String>,
    client: Option<Client>,
    account_id: Option<String>,
    spx_id: Option<String>,
    conids_map: Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
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
        self.client = Some(
            ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        );
        match self.get_account_id() {
            Ok(account_id) => {
                self.account_id = Some(account_id);
            }
            Err(e) => {
                eprintln!("Failed to get account ID: {}", e);
                exit(1)
            }
        }
        match self.get_spx_conid() {
            Ok(spx_id) => {
                self.spx_id = Some(spx_id);
            }
            Err(e) => {
                eprintln!("Failed to get SPX ID: {}", e);
                exit(1)
            }
        }
        match self.get_conids_map(&dates_slice, &strike_slice) {
            Ok(conids_map) => Ok({
                self.conids_map = Some(conids_map);
            }),
            Err(e) => {
                eprintln!("Failed to get SPX ID: {}", e);
                exit(1)
            }
        }
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
            .header("User-Agent", "trading_bot_rust/1.0")
            .send()?;

        if !response.status().is_success() {
            eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
            exit(1);
        }

        let account_result: Vec<AccountResponse> = response.json()?;
        if let Some(first_account) = account_result.get(0) {
            return Ok(first_account.id.clone());
        } else {
            eprintln!("No account found in the response");
            exit(1);
        }
    }

    // Function that sends a GET request for SPX ID
    fn get_spx_conid(&self) -> Result<String, Box<dyn Error>> {
        let search_url: String = format!(
            "https://{}:{}/v1/api/iserver/secdef/search?symbol=SPX",
            self.domain.as_ref().unwrap(),
            self.port.as_ref().unwrap()
        );

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(&search_url)
            .header("Connection", "keep-alive")
            .header("User-Agent", "trading_bot_rust/1.0")
            .send()?;

        if !response.status().is_success() {
            eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
            exit(1);
        }

        let search_results: Vec<SecDefResponse> = response.json()?;

        for result in &search_results {
            if let Some(conid) = &result.conid {
                if result.company_name == "S&P 500 Stock Index" && !conid.is_empty() {
                    return Ok(conid.to_string());
                }
            }
        }

        eprintln!("No SPX conid found in the response");
        exit(1);
    }

    // Function that gets a list of conids for all relevant contracts
    fn get_conids_map(
        &self,
        dates_slice: &[String],
        strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
    ) -> Result<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>, Box<dyn Error>>
    {
        let mut conids_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
            HashMap::new();
        let mut months_slice: Vec<String> = Vec::new();

        for date in dates_slice {
            conids_map.insert(
                date.clone(),
                ["C", "P"]
                    .iter()
                    .map(|&opt_type| {
                        let strikes: Vec<f64> = strike_slice
                            .get(date)
                            .and_then(|m| m.get(opt_type))
                            .cloned()
                            .unwrap_or_else(Vec::new);
                        (
                            opt_type.to_string(),
                            strikes
                                .into_iter()
                                .map(|s| (OrderedFloat(s), String::new()))
                                .collect(),
                        )
                    })
                    .collect(),
            );
        }

        for date in dates_slice {
            let formatted_date: String = convert_date(date);
            if !months_slice.contains(&formatted_date) {
                months_slice.push(formatted_date);
            }
        }

        for month in months_slice {
            let search_url: String = format!(
                "https://{}:{}/v1/api/iserver/secdef/info?conid={}&sectype=OPT&month={}&exchange=SMART&strike=0",
                self.domain.as_ref().unwrap(),
                self.port.as_ref().unwrap(),
                self.spx_id.as_ref().unwrap(),
                month
            );

            let response: Response = self
                .client
                .as_ref()
                .ok_or("Client is not initialized")?
                .get(&search_url)
                .header("Connection", "keep-alive")
                .header("User-Agent", "trading_bot_rust/1.0")
                .send()?;

            if !response.status().is_success() {
                eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
                exit(1);
            }

            let search_results: Vec<SecDefInfoResponse> = response.json()?;

            for contract in &search_results {
                if contract.trading_class == "SPXW" {
                    let date: &str = &contract.maturity_date[2..];
                    if let Some(date_map) = conids_map.get_mut(date) {
                        if let Some(opt_type_map) = date_map.get_mut(&contract.right) {
                            if let Some(conid_place) =
                                opt_type_map.get_mut((&contract.strike).into())
                            {
                                *conid_place = contract.conid.to_string();
                            }
                        }
                    }
                }
            }

            sleep(Duration::from_secs(1));
        }

        Ok(conids_map)
    }
}
