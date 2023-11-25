use ordered_float::OrderedFloat;
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    header::CONTENT_TYPE,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    error::Error,
    io::{self, ErrorKind},
    process::exit,
};

use crate::{
    helpers::{build_request_data, log_error, log_message},
    structs::{
        AccountResponse, Confirmation, Contender, PortfolioResponse, RequestDataStruct,
        SecDefInfoResponse, SecDefResponse,
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

    //   dates_slice: Vec<String>,
    //   strike_slice: HashMap<String, HashMap<String, Vec<f64>>>,

    pub(crate) fn init(
        &mut self,
        current_price: f64,
        discount_value: f64,
        domain: String,
        port: String,
        num_days: i64,
    ) -> Result<(), Box<dyn Error>> {
        let mut current_month: String = String::new();
        let mut next_month: String = String::new();

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
            Err(e) => log_error(format!("Failed to get account ID: {}", e)),
        }
        match self.get_spx_conid() {
            Ok((spx_id, month1, month2)) => {
                self.spx_id = Some(spx_id);
                current_month = month1;
                next_month = month2;
            }
            Err(e) => log_error(format!("Failed to get SPX ID: {}", e)),
        }

        exit(1);
        match self.get_conids_map(current_price, num_days) {
            Ok(conids_map) => Ok({
                self.conids_map = Some(conids_map);
            }),
            Err(e) => {
                log_error(format!("Failed to init conid map: {}", e));
                exit(1);
            }
        }
    }

    // TODO: INIT DATES SLICES AND STRIKES SLICES AS WELL, FIX CONID INIT

    // Function that gets a list of conids for all relevant contracts.
    fn get_conids_map(
        &self,
        current_price: f64,
        num_days: i64,
    ) -> Result<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>, Box<dyn Error>>
    {
        let mut conids_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
            HashMap::new();
        let months_slice: Vec<String> = Vec::new();

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
                log_error(format!(
                    "{}\nBody: {:?}",
                    response.status(),
                    response.text()?
                ));
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

    /*
     *
     *
     *
     *
     *
     */

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
            log_error(format!(
                "{}\nBody: {:?}",
                response.status(),
                response.text()?
            ));
            exit(1);
        }

        let account_result: Vec<AccountResponse> = response.json()?;
        if let Some(first_account) = account_result.get(0) {
            return Ok(first_account.id.clone());
        } else {
            log_error(format!("No account found in the response"));
            exit(1);
        }
    }

    // Function that sends a GET request for SPX ID.
    fn get_spx_conid(&self) -> Result<(String, String, String), Box<dyn Error>> {
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
            log_error(format!(
                "{}\nBody: {:?}",
                response.status(),
                response.text()?
            ));
            exit(1);
        }

        let search_results: Vec<SecDefResponse> = response.json()?;
        let mut month1: String = String::new();
        let mut month2: String = String::new();

        for result in &search_results {
            if let Some(conid) = &result.conid {
                if result.company_name == "S&P 500 Stock Index" && !conid.is_empty() {
                    if let Some(sections) = &result.sections {
                        for section in sections {
                            if section.sec_type == "OPT" {
                                if let Some(months) = &section.months {
                                    let months_vec: Vec<&str> = months.split(';').collect();
                                    if months_vec.len() >= 2 {
                                        month1 = months_vec[0].to_string();
                                        month2 = months_vec[1].to_string();
                                    }
                                }
                                break;
                            }
                        }
                    }

                    return Ok((conid.to_string(), month1, month2));
                }
            }
        }

        log_error(format!("No SPX conid found in the response"));
        exit(1);
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
            log_error(format!(
                "{}\nBody: {:?}",
                response.status(),
                response.text()?
            ));
            exit(1);
        }

        let search_results: PortfolioResponse = response.json()?;
        Ok(search_results.equity_with_loan_value.amount)
    }

    // Function that cancels all submitted and presubmitted orders.
    pub(crate) fn cancel_pending_orders(&mut self) {
        log_message(format!("Cancelling all pending limit orders."));

        if let Some(live_orders) = &self.live_orders {
            let order_ids: Vec<String> = live_orders.iter().cloned().collect();

            for order_id in order_ids {
                match self.cancel_order(&order_id) {
                    Ok(message) => log_message(format!("{}.", message)),
                    Err(e) => log_message(format!("{}.", e)),
                }
            }
        }

        if let Some(live_orders) = &mut self.live_orders {
            live_orders.clear();
        }

        log_message(format!("All pending limit orders cancelled."));
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
            Ok(format!("Order ID {} cancelled successfully", order_id))
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
        &mut self,
        contender_contracts: &Vec<Contender>,
        num_fills: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let order_url: String = format!(
            "{}/v1/api/iserver/account/{}/orders",
            self.base_url.as_ref().unwrap(),
            self.account_id.as_ref().unwrap()
        );

        let request_data: RequestDataStruct = build_request_data(
            contender_contracts,
            num_fills,
            &self.account_id,
            &self.conids_map,
            self.discount_value,
        );

        // Serialize the request data to JSON, handle possible serialization error.
        let json_data: Vec<u8> = serde_json::to_vec(&request_data)?;

        // Make the post request with the serialized JSON data.
        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .post(&order_url)
            .header(CONTENT_TYPE, "application/json")
            .header("Connection", "keep-alive")
            .header("User-Agent", "trading_bot_rust/1.0")
            .body(json_data)
            .send()?;

        if !response.status().is_success() {
            log_error(format!(
                "{}\nBody: {:?}",
                response.status(),
                response.text()?
            ));
            exit(1);
        }

        let mut generic_responses: Vec<Value> = response.json()?;

        loop {
            if let Some(confirm_id) = generic_responses[0]["id"].as_str() {
                let confirm_url = format!(
                    "{}/v1/api/iserver/reply/{}",
                    self.base_url.as_ref().unwrap(),
                    confirm_id
                );
                let confirm_data: Confirmation = Confirmation { confirmed: true };

                let json_data_confirm: Vec<u8> = serde_json::to_vec(&confirm_data)?;
                let confirm_response: Response = self
                    .client
                    .as_ref()
                    .ok_or("Client is not initialized")?
                    .post(&confirm_url)
                    .header(CONTENT_TYPE, "application/json")
                    .header("Connection", "keep-alive")
                    .header("User-Agent", "trading_bot_rust/1.0")
                    .body(json_data_confirm)
                    .send()?;

                if confirm_response.status().is_success() {
                    generic_responses = confirm_response.json()?;
                } else {
                    log_error(format!(
                        "{}\nBody: {:?}",
                        confirm_response.status(),
                        confirm_response.text()?
                    ));
                    exit(1);
                }
            } else if generic_responses[0].get("order_id").is_some() {
                if let Some(live_orders) = &mut self.live_orders {
                    for order in &generic_responses {
                        if let Some(order_id) = order["order_id"].as_str() {
                            live_orders.push(order_id.to_string());
                        }
                    }
                }
                break;
            } else {
                break;
            }
        }
        Ok(())
    }
}
