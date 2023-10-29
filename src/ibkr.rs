use ordered_float::OrderedFloat;
use reqwest::blocking::{Client, ClientBuilder, Response};
use std::{
    collections::HashMap,
    error::Error,
    io::{self, ErrorKind},
    process::exit,
    thread::sleep,
    time::Duration,
};

use crate::{
    helpers::{build_request_data, generate_conids_structure, generate_months_slice},
    structs::{
        AccountResponse, Contender, OrderBody, PortfolioResponse, SecDefInfoResponse,
        SecDefResponse,
    },
};

pub(crate) struct IBKR {
    discount_value: Option<f64>,
    base_url: Option<String>,
    live_orders: Option<Vec<String>>,
    client: Option<Client>,
    account_id: Option<String>,
    spx_id: Option<String>,
    conids_map: Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
}

impl IBKR {
    pub(crate) fn new() -> Self {
        IBKR {
            discount_value: None,
            base_url: None,
            live_orders: None,
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
        self.base_url = Some(format!("https://{}:{}", domain, port));
        self.live_orders = Some(Vec::new());
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
                exit(1);
            }
        }
        match self.get_spx_conid() {
            Ok(spx_id) => {
                self.spx_id = Some(spx_id);
            }
            Err(e) => {
                eprintln!("Failed to get SPX ID: {}", e);
                exit(1);
            }
        }
        match self.get_conids_map(&dates_slice, &strike_slice) {
            Ok(conids_map) => Ok({
                self.conids_map = Some(conids_map);
            }),
            Err(e) => {
                eprintln!("Failed to init conid map: {}", e);
                exit(1);
            }
        }
    }

    // Function that sends a GET request for portfolio ID.
    fn get_account_id(&self) -> Result<String, Box<dyn Error>> {
        let search_url: String = format!(
            "{}/v1/api/portfolio/accounts",
            self.base_url.as_ref().unwrap()
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

    // Function that sends a GET request for SPX ID.
    fn get_spx_conid(&self) -> Result<String, Box<dyn Error>> {
        let search_url: String = format!(
            "{}/v1/api/iserver/secdef/search?symbol=SPX",
            self.base_url.as_ref().unwrap()
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

    // Function that gets a list of conids for all relevant contracts.
    fn get_conids_map(
        &self,
        dates_slice: &[String],
        strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
    ) -> Result<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>, Box<dyn Error>>
    {
        let mut conids_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
            generate_conids_structure(dates_slice, strike_slice);
        let months_slice: Vec<String> = generate_months_slice(dates_slice);

        for month in months_slice {
            let search_url: String = format!(
                "{}/v1/api/iserver/secdef/info?conid={}&sectype=OPT&month={}&exchange=SMART&strike=0",
                self.base_url.as_ref().unwrap(),
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
        }

        Ok(conids_map)
    }

    // Function that sends a GET request for portfolio value.
    pub(crate) fn get_portfolio_value(&self) -> Result<f64, Box<dyn Error>> {
        let search_url: String = format!(
            "{}/v1/api/portfolio/{}/summary",
            self.base_url.as_ref().unwrap(),
            self.account_id.as_ref().unwrap()
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

        let search_results: PortfolioResponse = response.json()?;
        Ok(search_results.equity_with_loan_value.amount)
    }

    // Function that cancels all submitted and presubmitted orders.
    pub(crate) fn cancel_pending_orders(&mut self) {
        println!("Cancelling all pending limit orders");

        if let Some(live_orders) = &self.live_orders {
            let order_ids: Vec<String> = live_orders.iter().cloned().collect();

            for order_id in order_ids {
                match self.cancel_order(&order_id) {
                    Ok(message) => println!("{}", message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }

        if let Some(live_orders) = &mut self.live_orders {
            live_orders.clear();
        }

        sleep(Duration::from_secs(1));
    }

    // Function that cancels a single order.
    fn cancel_order(&self, order_id: &str) -> Result<String, Box<dyn Error>> {
        let cancel_order_url: String = format!(
            "{}/v1/api/iserver/account/{}/order/{}",
            self.base_url.as_ref().unwrap(),
            self.account_id.as_ref().unwrap(),
            order_id
        );

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .delete(&cancel_order_url)
            .header("Connection", "keep-alive")
            .header("User-Agent", "trading_bot_rust/1.0")
            .send()?;

        if response.status().is_success() {
            Ok(format!("Order ID {} cancelled successfully.", order_id))
        } else {
            Err(Box::new(io::Error::new(
                ErrorKind::Other,
                format!(
                    "Failed to cancel order ID {}. HTTP status: {}",
                    order_id,
                    response.status()
                ),
            )))
        }
    }

    // Function that makes orders all contender contracts.
    pub(crate) fn order_contender_contracts(
        &self,
        contender_contracts: &Vec<Contender>,
        num_fills: i32,
    ) {
        let _order_url: String = format!(
            "{}/v1/api/iserver/account/{}/orders",
            self.base_url.as_ref().unwrap(),
            self.account_id.as_ref().unwrap()
        );

        let _request_data: Vec<OrderBody> = build_request_data(
            contender_contracts,
            num_fills,
            &self.account_id,
            &self.conids_map,
            self.discount_value,
        );
    }
}
