use chrono::{Datelike, Local};
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
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    helpers::{calc_rank_value, calc_time_difference, calendar_spread_risk_free_profit},
    logging::{log_error, log_message},
    orders::build_request_data,
    structs::{
        AccountResponse, Confirmation, Contender, Contract, MarketDataResponse, Opt,
        PortfolioResponse, RequestDataStruct, SecDefInfoResponse, SecDefResponse,
    },
};

/// Enum representing option types for different strategies.
enum OptionType {
    Calendar,
    Butterfly,
    BoxSpread,
    All,
}

impl OptionType {
    /// Converts a string to an `OptionType`.
    ///
    /// # Arguments
    ///
    /// * `s` - A string representing the option type.
    ///
    /// # Returns
    ///
    /// An `Option<Self>` representing the converted option type.
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(OptionType::Calendar),
            "2" => Some(OptionType::Butterfly),
            "3" => Some(OptionType::BoxSpread),
            _ => Some(OptionType::All),
        }
    }
}

/// Struct representing the IBKR client, including configuration and internal state.
pub(crate) struct IBKR {
    ticker: Option<String>,
    discount_value: Option<f64>,
    arb_val: Option<f64>,
    strike_dif_value: Option<f64>,
    base_url: Option<String>,
    live_orders: Option<Vec<String>>,
    client: Option<Client>,
    account_id: Option<String>,
    ticker_id: Option<String>,
    conids_strings: Option<Vec<String>>,
    dates_slice: Option<Vec<String>>,
    strike_slice: Option<HashMap<String, HashMap<String, Vec<f64>>>>,
    conids_map: Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
}

impl IBKR {
    /// Creates a new `IBKR` instance.
    ///
    /// # Returns
    ///
    /// A new `IBKR` struct with uninitialized fields.
    pub(crate) fn new() -> Self {
        IBKR {
            ticker: None,
            discount_value: None,
            arb_val: None,
            strike_dif_value: None,
            base_url: None,
            live_orders: None,
            client: None,
            account_id: None,
            ticker_id: None,
            conids_strings: None,
            dates_slice: None,
            strike_slice: None,
            conids_map: None,
        }
    }

    /// Initializes the IBKR client with necessary configurations and retrieves required data.
    ///
    /// # Arguments
    ///
    /// * `ticker` - The stock ticker to trade.
    /// * `discount_value` - The discount value applied to orders.
    /// * `arb_val` - Arbitrage value threshold.
    /// * `strike_dif_value` - Difference in strike price for specific strategies.
    /// * `domain` - The domain for IBKR API.
    /// * `port` - The port for IBKR API.
    /// * `num_days` - Number of days to consider for option expiry.
    /// * `num_days_offset` - Number of days to offset from current day.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the initialization succeeded or failed.
    pub(crate) fn init(
        &mut self,
        ticker: String,
        discount_value: f64,
        arb_val: f64,
        strike_dif_value: f64,
        domain: String,
        port: String,
        num_days: i64,
        num_days_offset: i64,
    ) -> Result<(), Box<dyn Error>> {
        let mut current_month: String = String::new();
        let mut next_month: String = String::new();

        self.ticker = Some(ticker);
        self.discount_value = Some(discount_value);
        self.arb_val = Some(arb_val);
        self.strike_dif_value = Some(strike_dif_value);
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
        match self.get_ticker_conid() {
            Ok((ticker_id, month1, month2)) => {
                self.ticker_id = Some(ticker_id);
                current_month = month1;
                next_month = month2;
            }
            Err(e) => log_error(format!("Failed to get ticker ID: {}", e)),
        }

        match self.get_conids_map(num_days, num_days_offset, current_month, next_month) {
            Ok((conids_strings, dates_slice, strike_slice, conids_map)) => {
                self.conids_strings = Some(conids_strings);
                self.dates_slice = Some(dates_slice);
                self.strike_slice = Some(strike_slice);
                self.conids_map = Some(conids_map);
            }
            Err(e) => {
                log_error(format!("Failed to init conid map: {}", e));
                exit(1);
            }
        }

        self.init_ticker_data()?;

        Ok(())
    }

    /// Retrieves contender contracts based on the specified option type.
    ///
    /// # Arguments
    ///
    /// * `option` - The type of option strategy.
    /// * `num_orders` - The number of contender contracts to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of contender contracts or an error.
    pub(crate) fn get_contender_contracts(
        &self,
        option: &str,
        num_orders: i32,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let contracts_map: HashMap<String, Opt> = self.get_ticker_data()?;
        let mut contender_contracts_total: Vec<Contender> = Vec::new();

        let dates_slice: &Vec<String> =
            self.dates_slice.as_ref().ok_or("dates slice is not set")?;
        let strike_slice: &HashMap<String, HashMap<String, Vec<f64>>> = self
            .strike_slice
            .as_ref()
            .ok_or("strike slice is not set")?;
        let conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
            self.conids_map.as_ref().ok_or("conids map is not set")?;

        match OptionType::from_str(option).ok_or("Invalid option type")? {
            OptionType::Calendar => {
                contender_contracts_total.extend(self.get_calendar_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
            }
            OptionType::Butterfly => {
                contender_contracts_total.extend(self.get_butterfly_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
            }
            OptionType::BoxSpread => {
                contender_contracts_total.extend(self.get_boxspread_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
            }
            OptionType::All => {
                contender_contracts_total.extend(self.get_calendar_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
                contender_contracts_total.extend(self.get_butterfly_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
                contender_contracts_total.extend(self.get_boxspread_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                    conids_map,
                )?);
            }
        }

        contender_contracts_total.sort_by(|a, b| b.rank_value.partial_cmp(&a.rank_value).unwrap());

        let num_orders_usize: usize = num_orders as usize;
        if contender_contracts_total.len() > num_orders_usize {
            contender_contracts_total.truncate(num_orders_usize);
        }

        Ok(contender_contracts_total)
    }

    /// Retrieves ticker data from the IBKR API and parses it into a map of options.
    ///
    /// # Returns
    ///
    /// A `Result` containing a map of options or an error.
    fn get_ticker_data(&self) -> Result<HashMap<String, Opt>, Box<dyn Error>> {
        let mut contracts_map: HashMap<String, Opt> = HashMap::new();
        let chain_url: String = format!(
            "{}/v1/api/iserver/marketdata/snapshot",
            self.base_url.as_ref().unwrap()
        );
        let conids_arr: &Vec<String> = self.conids_strings.as_ref().unwrap();

        let client: Arc<Client> = Arc::new(
            self.client
                .as_ref()
                .ok_or("Client is not initialized")?
                .clone(),
        );
        let chain_url: Arc<String> = Arc::new(chain_url);
        let response_arr: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(Vec::new()));

        let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

        for conid in conids_arr {
            let client: Arc<Client> = Arc::clone(&client);
            let chain_url: Arc<String> = Arc::clone(&chain_url);
            let response_arr: Arc<Mutex<Vec<Response>>> = Arc::clone(&response_arr);
            let conid: String = conid.clone();

            let handle: thread::JoinHandle<()> = thread::spawn(move || {
                let params: [(&str, &str); 2] = [("conids", &conid), ("fields", "84,85,86")];

                match client
                    .get(chain_url.as_ref())
                    .header("Connection", "keep-alive")
                    .header("User-Agent", "trading_bot_rust/1.0")
                    .query(&params)
                    .send()
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            let mut response_arr: std::sync::MutexGuard<'_, Vec<Response>> =
                                response_arr.lock().unwrap();
                            response_arr.push(response);
                        } else {
                            log_error(format!(
                                "{}\nBody: {:?}",
                                response.status(),
                                response.text().unwrap_or_else(|_| "".to_string())
                            ));
                        }
                    }
                    Err(e) => log_error(format!("Failed to get ticker data: {}", e)),
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut response_vec: std::sync::MutexGuard<'_, Vec<Response>> =
            response_arr.lock().unwrap();

        for response in response_vec.drain(..) {
            let generic_responses: Vec<MarketDataResponse> = response.json()?;

            for response in &generic_responses {
                if let Some(field_84_value) = &response.field_84 {
                    if let Some(field_85_value) = &response.field_85 {
                        if let Some(field_86_value) = &response.field_86 {
                            if field_84_value != "" && field_85_value != "" && field_86_value != ""
                            {
                                let conid: &String = &response.conid_ex;
                                let bid_val: f64 = field_84_value
                                    .replace(",", "")
                                    .parse::<f64>()
                                    .map_err(|_| "Failed to parse bid")?;
                                let ask_val: f64 = field_86_value
                                    .replace(",", "")
                                    .parse::<f64>()
                                    .map_err(|_| "Failed to parse ask")?;
                                let asz_val: f64 = field_85_value
                                    .replace(",", "")
                                    .parse::<f64>()
                                    .map_err(|_| "Failed to parse asz")?;

                                let mkt_val: f64 =
                                    ((bid_val + ask_val) / 2.0 * 100.0).round() / 100.0;

                                contracts_map.insert(
                                    conid.to_string(),
                                    Opt {
                                        asz: asz_val,
                                        mkt: mkt_val,
                                        bid: bid_val,
                                    },
                                );
                            } else {
                                let conid: &String = &response.conid_ex;
                                contracts_map.insert(
                                    conid.to_string(),
                                    Opt {
                                        asz: 0.0,
                                        mkt: 0.0,
                                        bid: 0.0,
                                    },
                                );
                            }
                        } else {
                            let conid: &String = &response.conid_ex;
                            contracts_map.insert(
                                conid.to_string(),
                                Opt {
                                    asz: 0.0,
                                    mkt: 0.0,
                                    bid: 0.0,
                                },
                            );
                        }
                    } else {
                        let conid: &String = &response.conid_ex;
                        contracts_map.insert(
                            conid.to_string(),
                            Opt {
                                asz: 0.0,
                                mkt: 0.0,
                                bid: 0.0,
                            },
                        );
                    }
                } else {
                    let conid: &String = &response.conid_ex;
                    contracts_map.insert(
                        conid.to_string(),
                        Opt {
                            asz: 0.0,
                            mkt: 0.0,
                            bid: 0.0,
                        },
                    );
                }
            }
        }

        return Ok(contracts_map);
    }

    /// Initializes ticker data by sending a request to the IBKR API.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the initialization succeeded or failed.
    fn init_ticker_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let chain_url: String = format!(
            "{}/v1/api/iserver/marketdata/snapshot",
            self.base_url.as_ref().unwrap()
        );

        let conids_arr: &Vec<String> = self.conids_strings.as_ref().unwrap();

        for conid in conids_arr {
            let params: [(&str, &str); 2] = [("conids", conid), ("fields", "84,85,86")];

            let response: Response = self
                .client
                .as_ref()
                .ok_or("Client is not initialized")?
                .get(chain_url.clone())
                .header("Connection", "keep-alive")
                .header("User-Agent", "trading_bot_rust/1.0")
                .query(&params)
                .send()?;

            if !response.status().is_success() {
                log_error(format!(
                    "{}\nBody: {:?}",
                    response.status(),
                    response.text()?
                ));
                exit(1);
            }
        }

        Ok(())
    }

    /// Retrieves contender contracts for calendar spreads.
    ///
    /// # Arguments
    ///
    /// * `contracts_map` - A map of options.
    /// * `dates_slice` - A slice of relevant dates for the options.
    /// * `strike_slice` - A map of strike prices.
    /// * `conids_map` - A map of conids for the contracts.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of calendar contenders or an error.
    pub(crate) fn get_calendar_contenders(
        &self,
        contracts_map: &HashMap<String, Opt>,
        dates_slice: &Vec<String>,
        strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
        conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let arb_threshold: f64 = 0.0 + self.arb_val.as_ref().unwrap();
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let date_for_mean: &String = &dates_slice[0];
        let mut mean_val: f64 = 0.0;
        if let Some(strike_data) = strike_slice.get(date_for_mean) {
            let sum: f64 = strike_data["C"].iter().sum();
            let count: f64 = strike_data["C"].len() as f64;
            mean_val = sum / count;
        }

        for date_index in 0..(dates_slice.len() - 1) {
            let date: &String = &dates_slice[date_index];

            if let Some(strike_data) = strike_slice.get(date) {
                for (contract_type, strikes) in strike_data.iter() {
                    for current_strike in strikes {
                        let current_contract_conid: &String = conids_map
                            .get(date)
                            .and_then(|ct| ct.get(contract_type))
                            .and_then(|ct| ct.get(current_strike.into()))
                            .ok_or("Error accessing current conid")?;
                        let current_opt: &Opt = contracts_map
                            .get(current_contract_conid)
                            .ok_or("Error accessing current contract")?;

                        let next_date: &String = &dates_slice[date_index + 1];
                        let next_contract_conid: Option<&String> = conids_map
                            .get(next_date)
                            .and_then(|ct| ct.get(contract_type))
                            .and_then(|ct| ct.get(current_strike.into()));

                        if let Some(next_contract_conid) = next_contract_conid {
                            let next_opt: &Opt = contracts_map
                                .get(next_contract_conid)
                                .ok_or("Error accessing next contract")?;

                            let arb_val: f64 = current_opt.mkt - next_opt.mkt;

                            if arb_val >= arb_threshold
                                && current_opt.bid > 1.0
                                && next_opt.bid > 1.0
                                && current_opt.asz > 0.0
                                && next_opt.asz > 0.0
                                && calc_time_difference(date, next_date) == 1
                                && calendar_spread_risk_free_profit(current_strike, arb_val) > 0.25
                                && (current_strike - mean_val).abs() <= 500.0
                            {
                                let avg_ask: f64 = ((current_opt.asz + next_opt.asz) / 2.0).round();
                                let rank_value: f64 =
                                    calc_rank_value(avg_ask, arb_val, &current_date, date);

                                contender_contracts.push(Contender {
                                    arb_val: (arb_val * 100.0).round() / 100.0,
                                    avg_ask,
                                    type_spread: "Calendar".to_string(),
                                    exp_date: date.clone(),
                                    rank_value,
                                    contracts: vec![
                                        Contract {
                                            strike: *current_strike,
                                            mkt_price: current_opt.mkt,
                                            date: date.clone(),
                                            type_contract: contract_type.clone(),
                                        },
                                        Contract {
                                            strike: *current_strike,
                                            mkt_price: next_opt.mkt,
                                            date: next_date.clone(),
                                            type_contract: contract_type.clone(),
                                        },
                                    ],
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(contender_contracts)
    }

    /// Retrieves contender contracts for butterfly spreads.
    ///
    /// # Arguments
    ///
    /// * `contracts_map` - A map of options.
    /// * `dates_slice` - A slice of relevant dates for the options.
    /// * `strike_slice` - A map of strike prices.
    /// * `conids_map` - A map of conids for the contracts.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of butterfly contenders or an error.
    pub(crate) fn get_butterfly_contenders(
        &self,
        contracts_map: &HashMap<String, Opt>,
        dates_slice: &Vec<String>,
        strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
        conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let arb_threshold: f64 = 0.0 + self.arb_val.as_ref().unwrap();
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                for &contract_type in &["C", "P"] {
                    if let Some(contract_strikes) = strike_data.get(contract_type) {
                        if contract_strikes.len() > 2 {
                            for i in 1..(contract_strikes.len() - 1) {
                                let current_strike: &f64 = &contract_strikes[i];
                                let current_contract_conid: &String = conids_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(current_strike.into()))
                                    .ok_or("Error accessing current conid")?;
                                let current_contract: &Opt = contracts_map
                                    .get(current_contract_conid)
                                    .ok_or("Error accessing current contract")?;

                                let left_strike: &f64 = &contract_strikes[i - 1];
                                let left_contract_conid: &String = conids_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(left_strike.into()))
                                    .ok_or("Error accessing left conid")?;
                                let left_contract: &Opt = contracts_map
                                    .get(left_contract_conid)
                                    .ok_or("Error accessing left contract")?;

                                let right_strike: &f64 = &contract_strikes[i + 1];
                                let right_contract_conid: &String = conids_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(right_strike.into()))
                                    .ok_or("Error accessing right conid")?;
                                let right_contract: &Opt = contracts_map
                                    .get(right_contract_conid)
                                    .ok_or("Error accessing right contract")?;

                                let arb_val: f64 = (2.0 * current_contract.mkt)
                                    - (left_contract.mkt + right_contract.mkt);

                                if arb_val >= arb_threshold
                                    && left_contract.bid > 1.0
                                    && right_contract.bid > 1.0
                                    && current_contract.bid > 1.0
                                    && left_contract.asz > 0.0
                                    && right_contract.asz > 0.0
                                    && current_contract.asz > 0.0
                                    && ((current_strike - left_strike) * 10.0).round() / 10.0
                                        == self.strike_dif_value.unwrap()
                                    && ((right_strike - current_strike) * 10.0).round() / 10.0
                                        == self.strike_dif_value.unwrap()
                                {
                                    let avg_ask: f64 = ((left_contract.asz
                                        + right_contract.asz
                                        + (2.0 * current_contract.asz))
                                        / 4.0)
                                        .round();
                                    let rank_value: f64 =
                                        calc_rank_value(avg_ask, arb_val, &current_date, date);

                                    contender_contracts.push(Contender {
                                        arb_val: (arb_val * 100.0).round() / 100.0,
                                        avg_ask,
                                        type_spread: "Butterfly".to_string(),
                                        exp_date: date.clone(),
                                        rank_value,
                                        contracts: vec![
                                            Contract {
                                                strike: *left_strike,
                                                mkt_price: left_contract.mkt,
                                                date: date.clone(),
                                                type_contract: contract_type.to_string(),
                                            },
                                            Contract {
                                                strike: *current_strike,
                                                mkt_price: current_contract.mkt,
                                                date: date.clone(),
                                                type_contract: contract_type.to_string(),
                                            },
                                            Contract {
                                                strike: *right_strike,
                                                mkt_price: right_contract.mkt,
                                                date: date.clone(),
                                                type_contract: contract_type.to_string(),
                                            },
                                        ],
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(contender_contracts)
    }

    /// Retrieves contender contracts for box spread strategies.
    ///
    /// # Arguments
    ///
    /// * `contracts_map` - A map of options.
    /// * `dates_slice` - A slice of relevant dates for the options.
    /// * `strike_slice` - A map of strike prices.
    /// * `conids_map` - A map of conids for the contracts.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of box spread contenders or an error.
    pub(crate) fn get_boxspread_contenders(
        &self,
        contracts_map: &HashMap<String, Opt>,
        dates_slice: &Vec<String>,
        strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
        conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let arb_threshold: f64 = -5.0 - self.arb_val.as_ref().unwrap();
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                if let (Some(cs), Some(ps)) = (strike_data.get("C"), strike_data.get("P")) {
                    if cs.len() > 1 && ps.len() > 1 {
                        for i in 0..(cs.len() - 1) {
                            let current_strike_c: &f64 = &cs[i];
                            let current_c_conid: &String = conids_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(current_strike_c.into()))
                                .ok_or("Error accessing current call conid")?;
                            let current_c: &Opt = contracts_map
                                .get(current_c_conid)
                                .ok_or("Error accessing current call contract")?;

                            let current_strike_p: &f64 = &ps[i];
                            let current_p_conid: &String = conids_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(current_strike_p.into()))
                                .ok_or("Error accessing current put conid")?;
                            let current_p: &Opt = contracts_map
                                .get(current_p_conid)
                                .ok_or("Error accessing current put contract")?;

                            let right_strike_c: &f64 = &cs[i + 1];
                            let right_c_conid: &String = conids_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(right_strike_c.into()))
                                .ok_or("Error accessing right call conid")?;
                            let right_c: &Opt = contracts_map
                                .get(right_c_conid)
                                .ok_or("Error accessing right call contract")?;

                            let right_strike_p: &f64 = &ps[i + 1];
                            let right_p_conid: &String = conids_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(right_strike_p.into()))
                                .ok_or("Error accessing right put conid")?;
                            let right_p: &Opt = contracts_map
                                .get(right_p_conid)
                                .ok_or("Error accessing right put contract")?;

                            let arb_val: f64 =
                                (current_p.mkt + right_c.mkt) - (current_c.mkt + right_p.mkt);

                            if arb_val <= arb_threshold
                                && current_c.bid > 1.0
                                && current_p.bid > 1.0
                                && right_c.bid > 1.0
                                && right_p.bid > 1.0
                                && current_c.asz > 0.0
                                && current_p.asz > 0.0
                                && right_c.asz > 0.0
                                && right_p.asz > 0.0
                                && ((right_strike_c - current_strike_c) * 10.0).round() / 10.0
                                    == self.strike_dif_value.unwrap()
                                && ((right_strike_p - current_strike_p) * 10.0).round() / 10.0
                                    == self.strike_dif_value.unwrap()
                            {
                                let avg_ask: f64 =
                                    ((current_c.asz + right_c.asz + current_p.asz + right_p.asz)
                                        / 4.0)
                                        .round();
                                let rank_value: f64 = calc_rank_value(
                                    avg_ask,
                                    (-1.0 * arb_val) - 5.0,
                                    &current_date,
                                    date,
                                );

                                contender_contracts.push(Contender {
                                    arb_val: (-1.0 * arb_val * 100.0).round() / 100.0,
                                    avg_ask,
                                    type_spread: "Boxspread".to_string(),
                                    exp_date: date.clone(),
                                    rank_value,
                                    contracts: vec![
                                        Contract {
                                            strike: *current_strike_p,
                                            mkt_price: current_p.mkt,
                                            date: date.clone(),
                                            type_contract: "P".to_string(),
                                        },
                                        Contract {
                                            strike: *current_strike_c,
                                            mkt_price: current_c.mkt,
                                            date: date.clone(),
                                            type_contract: "C".to_string(),
                                        },
                                        Contract {
                                            strike: *right_strike_c,
                                            mkt_price: right_c.mkt,
                                            date: date.clone(),
                                            type_contract: "C".to_string(),
                                        },
                                        Contract {
                                            strike: *right_strike_p,
                                            mkt_price: right_p.mkt,
                                            date: date.clone(),
                                            type_contract: "P".to_string(),
                                        },
                                    ],
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(contender_contracts)
    }

    /// Retrieves the account ID from the IBKR API.
    ///
    /// # Returns
    ///
    /// A `Result` containing the account ID or an error.
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

    /// Retrieves the ticker conid and relevant months for the options.
    ///
    /// # Returns
    ///
    /// A `Result` containing the ticker conid, current month, and next month or an error.
    fn get_ticker_conid(&self) -> Result<(String, String, String), Box<dyn Error>> {
        let search_url: String = format!(
            "{}/v1/api/iserver/secdef/search?symbol={}",
            self.base_url.as_ref().unwrap(),
            self.ticker.as_ref().unwrap()
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

        let result: &SecDefResponse = &search_results[0];
        if let Some(conid) = &result.conid {
            if !conid.is_empty() {
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

        log_error(format!("No ticker conid found in the response"));
        exit(1);
    }

    /// Retrieves the conid map for the options contracts.
    ///
    /// # Arguments
    ///
    /// * `num_days` - The number of days for which to retrieve conids.
    /// * `num_days_offset` - The offset for the number of days.
    /// * `current_month` - The current month for the options.
    /// * `next_month` - The next month for the options.
    ///
    /// # Returns
    ///
    /// A `Result` containing conids, dates, strike prices, and conid maps or an error.
    fn get_conids_map(
        &self,
        mut num_days: i64,
        num_days_offset: i64,
        current_month: String,
        next_month: String,
    ) -> Result<
        (
            Vec<String>,
            Vec<String>,
            HashMap<String, HashMap<String, Vec<f64>>>,
            HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
        ),
        Box<dyn Error>,
    > {
        let mut conids_strings: Vec<String> = Vec::new();
        let mut dates_slice: Vec<String> = Vec::new();
        let mut strike_slice: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        let mut conids_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
            HashMap::new();

        let search_url: String = format!(
            "{}/v1/api/iserver/secdef/info?conid={}&sectype=OPT&month={}&exchange=SMART&strike=0",
            self.base_url.as_ref().unwrap(),
            self.ticker_id.as_ref().unwrap(),
            current_month
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
        let current_date: String = Local::now().format("%y%m%d").to_string();
        let mut conids_str: String = String::new();
        let mut counter: i32 = 0;

        for sec_def_info in search_results.iter() {
            let type_opt: &String = &sec_def_info.right;
            let exp_date: String = sec_def_info
                .maturity_date
                .get(2..)
                .unwrap_or(&sec_def_info.maturity_date)
                .to_string();
            let strike: OrderedFloat<f64> = OrderedFloat(sec_def_info.strike);
            let conid: f64 = sec_def_info.conid;

            if calc_time_difference(&current_date, &exp_date) > (-1 + num_days_offset) {
                if !strike_slice.contains_key(&exp_date) {
                    num_days -= 1;
                    if num_days < 0 {
                        break;
                    }

                    dates_slice.push(exp_date.to_string());

                    strike_slice.insert(exp_date.to_string(), HashMap::new());
                    strike_slice
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("C".to_string(), Vec::new());
                    strike_slice
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("P".to_string(), Vec::new());

                    conids_map.insert(exp_date.to_string(), HashMap::new());
                    conids_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("C".to_string(), HashMap::new());
                    conids_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("P".to_string(), HashMap::new());
                }

                strike_slice
                    .get_mut(&exp_date)
                    .unwrap()
                    .get_mut(type_opt)
                    .unwrap()
                    .push(*strike);

                conids_map
                    .get_mut(&exp_date)
                    .unwrap()
                    .get_mut(type_opt)
                    .unwrap()
                    .insert(strike, conid.to_string());

                conids_str.push_str(&conid.to_string());
                conids_str.push_str(",");
                counter += 1;

                if counter == 300 {
                    conids_strings.push(conids_str);
                    conids_str = String::new();
                    counter = 0;
                }
            }
        }

        if num_days > 0 {
            let search_url_2: String = format!(
                "{}/v1/api/iserver/secdef/info?conid={}&sectype=OPT&month={}&exchange=SMART&strike=0",
                self.base_url.as_ref().unwrap(),
                self.ticker_id.as_ref().unwrap(),
                next_month
            );

            let response_2: Response = self
                .client
                .as_ref()
                .ok_or("Client is not initialized")?
                .get(&search_url_2)
                .header("Connection", "keep-alive")
                .header("User-Agent", "trading_bot_rust/1.0")
                .send()?;

            if !response_2.status().is_success() {
                log_error(format!(
                    "{}\nBody: {:?}",
                    response_2.status(),
                    response_2.text()?
                ));
                exit(1);
            }

            let search_results_2: Vec<SecDefInfoResponse> = response_2.json()?;

            for sec_def_info in search_results_2.iter() {
                let type_opt: &String = &sec_def_info.right;
                let exp_date: String = sec_def_info
                    .maturity_date
                    .get(2..)
                    .unwrap_or(&sec_def_info.maturity_date)
                    .to_string();
                let strike: OrderedFloat<f64> = OrderedFloat(sec_def_info.strike);
                let conid: f64 = sec_def_info.conid;

                if !strike_slice.contains_key(&exp_date) {
                    num_days -= 1;
                    if num_days < 0 {
                        break;
                    }

                    dates_slice.push(exp_date.to_string());

                    strike_slice.insert(exp_date.to_string(), HashMap::new());
                    strike_slice
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("C".to_string(), Vec::new());
                    strike_slice
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("P".to_string(), Vec::new());

                    conids_map.insert(exp_date.to_string(), HashMap::new());
                    conids_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("C".to_string(), HashMap::new());
                    conids_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .insert("P".to_string(), HashMap::new());
                }

                strike_slice
                    .get_mut(&exp_date)
                    .unwrap()
                    .get_mut(type_opt)
                    .unwrap()
                    .push(*strike);

                conids_map
                    .get_mut(&exp_date)
                    .unwrap()
                    .get_mut(type_opt)
                    .unwrap()
                    .insert(strike, conid.to_string());

                conids_str.push_str(&conid.to_string());
                conids_str.push_str(",");
                counter += 1;

                if counter == 300 {
                    conids_strings.push(conids_str);
                    conids_str = String::new();
                    counter = 0;
                }
            }
        }

        if !conids_str.is_empty() {
            conids_strings.push(conids_str);
        }

        for (_, strikes) in strike_slice.iter_mut() {
            strikes
                .get_mut("C")
                .unwrap()
                .sort_by(|a, b| a.partial_cmp(b).unwrap());
            strikes
                .get_mut("P")
                .unwrap()
                .sort_by(|a, b| a.partial_cmp(b).unwrap());
        }

        Ok((conids_strings, dates_slice, strike_slice, conids_map))
    }

    /// Retrieves the portfolio value from the IBKR API.
    ///
    /// # Returns
    ///
    /// A `Result` containing the portfolio value or an error.
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

    /// Cancels all pending limit orders for the account.
    ///
    /// This method iterates over all live orders stored in the `live_orders` field
    /// and calls the `cancel_order` method for each order. Once all orders are canceled,
    /// it clears the `live_orders` list and logs a success message.
    ///
    /// # Returns
    ///
    /// No return value, but logs messages indicating the result of each cancellation.
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

    /// Cancels a specific order by order ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The ID of the order to be canceled.
    ///
    /// # Returns
    ///
    /// A `Result` containing a message about the cancellation or an error.
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

    /// Places orders for the contender contracts.
    ///
    /// # Arguments
    ///
    /// * `contender_contracts` - A vector of contender contracts to be ordered.
    /// * `num_fills` - The number of fills for each contract.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the orders were successfully placed or not.
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

        let json_data: Vec<u8> = serde_json::to_vec(&request_data)?;

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
