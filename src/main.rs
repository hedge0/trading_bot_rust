use chrono::{Datelike, Local, NaiveDate, Timelike, Utc, Weekday};
use dotenv::dotenv;
use ordered_float::OrderedFloat;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
struct AccountResponse {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct SecDefResponse {
    company_name: String,
    con_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SecDefInfoResponse {
    con_id: f64,
    trading_class: String,
    maturity_date: String,
    right: String,
    strike: f64,
}

#[derive(Serialize, Deserialize)]
struct PortfolioResponse {
    equity_with_loan_value: PortfolioAmount,
}

#[derive(Serialize, Deserialize)]
struct PortfolioAmount {
    amount: f64,
}

#[derive(Serialize, Deserialize)]
struct OrderBody {
    acct_id: String,
    con_idex: String,
    order_type: String,
    listing_exchange: String,
    outside_rth: bool,
    price: f64,
    side: String,
    ticker: String,
    tif: String,
    referrer: String,
    quantity: i32,
    use_adaptive: bool,
}

#[derive(Serialize, Deserialize)]
struct RequestDataStruct {
    orders: Vec<OrderBody>,
}

#[derive(Serialize, Deserialize)]
struct Order {
    status: String,
    order_id: f64,
}

#[derive(Serialize, Deserialize)]
struct OrdersResponse {
    orders: Vec<Order>,
}

#[derive(Deserialize)]
struct AuthResponse {
    status: String,
    sessionid: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ChainResponse {
    rows: Vec<ChainRow>,
}

#[derive(Serialize, Deserialize)]
struct ChainRow {
    st: String,
    data: Vec<ChainData>,
    s: String,
}

#[derive(Serialize, Deserialize)]
struct ChainData {
    v: String,
}

#[derive(Serialize, Deserialize)]
struct Opt {
    asz: f64,
    mkt: f64,
    bid: f64,
}

#[derive(Serialize, Deserialize)]
struct Contract {
    strike: f64,
    mkt_price: f64,
    date: String,
    type_contract: String,
}

#[derive(Serialize, Deserialize)]
struct Contender {
    arb_val: f64,
    avg_ask: f64,
    type_spread: String,
    exp_date: String,
    rank_value: f64,
    contracts: Vec<Contract>,
}

impl Contender {
    fn action(&self, index: usize) -> &str {
        match self.type_spread.as_str() {
            "Calendar" => {
                if index == 0 {
                    "SELL"
                } else {
                    "UNKNOWN"
                }
            }
            "Butterfly" => {
                if index == 1 {
                    "SELL"
                } else {
                    "BUY "
                }
            }
            "Boxspread" => {
                if index % 2 == 1 {
                    "SELL"
                } else {
                    "BUY "
                }
            }
            _ => "UNKNOWN",
        }
    }

    fn multiplier(&self, num_fills: i32, index: usize) -> i32 {
        if self.type_spread == "Butterfly" && index == 1 {
            num_fills * 2
        } else {
            num_fills
        }
    }

    fn arb_value(&self) -> f64 {
        if self.type_spread == "Boxspread" {
            self.arb_val + 5.0
        } else {
            self.arb_val
        }
    }
}

enum OptionType {
    Calendar,
    Butterfly,
    BoxSpread,
    CalendarButterfly,
    CalendarBoxSpread,
    ButterflyBoxSpread,
    All,
}

impl OptionType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(OptionType::Calendar),
            "2" => Some(OptionType::Butterfly),
            "3" => Some(OptionType::BoxSpread),
            "4" => Some(OptionType::CalendarButterfly),
            "5" => Some(OptionType::CalendarBoxSpread),
            "6" => Some(OptionType::ButterflyBoxSpread),
            _ => Some(OptionType::All),
        }
    }
}

// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************

struct ActiveTick {
    username: Option<String>,
    password: Option<String>,
    apikey: Option<String>,
    num_days: Option<std::time::Duration>,
    client: Option<Client>,
    dates_slice: Option<Vec<String>>,
    strike_slice: Option<HashMap<String, HashMap<String, Vec<f64>>>>,
}

impl ActiveTick {
    fn new() -> Self {
        ActiveTick {
            username: None,
            password: None,
            apikey: None,
            num_days: None,
            client: None,
            dates_slice: None,
            strike_slice: None,
        }
    }

    fn init(
        &mut self,
        username: &str,
        password: &str,
        apikey: &str,
        num_days: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self.apikey = Some(apikey.to_string());
        self.num_days = Some(std::time::Duration::from_secs(num_days * 24 * 60 * 60));
        self.client = Some(Client::new());
        let session_id = self.get_session_id()?;
        let (dates, strikes) = self.get_spx_dates_and_strikes(&session_id)?;
        self.dates_slice = dates;
        self.strike_slice = strikes;
        Ok(())
    }

    // Function that returns datesSlice
    fn _get_dates_slice(&self) -> &Option<Vec<String>> {
        &self.dates_slice
    }

    // Function that returns strikeSlice
    fn _get_strike_slice(&self) -> &Option<HashMap<String, HashMap<String, Vec<f64>>>> {
        &self.strike_slice
    }

    // Function that checks if the user is authorized to use the Activetick API, and returns a sessionID if ok
    fn get_session_id(&self) -> Result<String, Box<dyn Error>> {
        let auth_url = "https://api.activetick.com/authorize.json";

        let params = [
            (
                "username",
                self.username.as_ref().ok_or("Missing username")?,
            ),
            (
                "password",
                self.password.as_ref().ok_or("Missing password")?,
            ),
            ("apikey", self.apikey.as_ref().ok_or("Missing apikey")?),
        ];

        let response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(auth_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Error: {}", response.status()).into());
        }

        let auth_results: AuthResponse = response.json()?;
        if auth_results.status == "ok" {
            if let Some(session_id) = auth_results.sessionid {
                return Ok(session_id);
            }
        } else {
            eprintln!("Error: User Unauthorized");
            exit(1);
        }

        Err("Failed to get session ID".into())
    }

    // Function that sends a GET request for SPX data, and then gets dates and strikes
    fn get_spx_dates_and_strikes(
        &self,
        session_id: &str,
    ) -> Result<
        (
            Option<Vec<String>>,
            Option<HashMap<String, HashMap<String, Vec<f64>>>>,
        ),
        Box<dyn Error>,
    > {
        let chain_url = "https://api.activetick.com/chain.json";
        let current_time = chrono::Local::now();
        let future_time = current_time + self.num_days.unwrap_or_default(); // handle None case
        let formatted_time = current_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let formatted_future_time = future_time.format("%Y-%m-%dT%H:%M:%S").to_string();

        let params = [
            ("sessionid", session_id),
            ("key", "SPXW_S U"),
            ("chaintype", "equity_options"),
            ("columns", "b,a,asz"),
            ("begin_maturity_time", &formatted_time),
            ("end_maturity_time", &formatted_future_time),
            ("ignore_empty", "false"),
        ];

        let response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(chain_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Error: {}", response.status()).into());
        }

        let chain_results: ChainResponse = response.json()?;
        if chain_results.rows.is_empty() {
            return Err("Error: SPX rows data is empty".into());
        }
        let mut dates_slice = Vec::new();
        let mut strike_slice = HashMap::new();

        for row in chain_results.rows.iter() {
            if row.st == "ok" {
                let parts: Vec<&str> = row.s.split('_').collect();
                let code = parts[1];
                let exp_date = &code[0..6];
                let type_opt = &code[6..7];
                let strike_str = &code[7..(code.len() - 3)];
                let strike: f64 = strike_str.parse().unwrap();

                if !strike_slice.contains_key(exp_date) {
                    dates_slice.push(exp_date.to_string());
                    strike_slice.insert(exp_date.to_string(), HashMap::new());
                    strike_slice
                        .get_mut(exp_date)
                        .unwrap()
                        .insert("C".to_string(), Vec::new());
                    strike_slice
                        .get_mut(exp_date)
                        .unwrap()
                        .insert("P".to_string(), Vec::new());
                }

                strike_slice
                    .get_mut(exp_date)
                    .unwrap()
                    .get_mut(type_opt)
                    .unwrap()
                    .push(strike);
            }
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

        Ok((Some(dates_slice), Some(strike_slice)))
    }

    // Function that sends a GET request for SPX data, and then parses the response
    fn get_spx_data(
        &self,
        session_id: &str,
    ) -> Result<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>, Box<dyn Error>>
    {
        let chain_url = "https://api.activetick.com/chain.json";
        let current_time = chrono::Local::now();
        let future_time = current_time + self.num_days.ok_or("num_days is not set")?;
        let formatted_time = current_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let formatted_future_time = future_time.format("%Y-%m-%dT%H:%M:%S").to_string();

        let params = [
            ("sessionid", session_id),
            ("key", "SPXW_S U"),
            ("chaintype", "equity_options"),
            ("columns", "b,a,asz"),
            ("begin_maturity_time", &formatted_time),
            ("end_maturity_time", &formatted_future_time),
            ("ignore_empty", "false"),
        ];

        let response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(chain_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Error: {}", response.status()).into());
        }

        let chain_results: ChainResponse = response.json()?;
        let mut contracts_map = HashMap::new();

        for row in chain_results.rows.iter() {
            if row.st == "ok" {
                let parts: Vec<&str> = row.s.split('_').collect();
                let code = parts[1];
                let exp_date = &code[0..6];
                let type_opt = &code[6..7];
                let strike_str = &code[7..(code.len() - 3)];
                let strike = OrderedFloat(strike_str.parse::<f64>().unwrap());
                let bid: f64 = row.data[0].v.parse().unwrap();
                let ask: f64 = row.data[1].v.parse().unwrap();
                let asz_val: f64 = row.data[2].v.parse().unwrap();
                let mkt_val = ((bid + ask) / 2.0).round();

                contracts_map
                    .entry(exp_date.to_string())
                    .or_insert_with(|| {
                        let mut m = HashMap::new();
                        m.insert("C".to_string(), HashMap::new());
                        m.insert("P".to_string(), HashMap::new());
                        m
                    })
                    .entry(type_opt.to_string())
                    .or_insert(HashMap::new())
                    .insert(
                        strike,
                        Opt {
                            asz: asz_val,
                            mkt: mkt_val,
                            bid: bid,
                        },
                    );
            }
        }

        Ok(contracts_map)
    }

    // Function that returns a slice of the top calendar arbs
    fn get_calendar_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts = Vec::new();
        let now = Local::now();
        let current_date = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice = self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date_index in 0..(dates_slice.len() - 1) {
            let date = &dates_slice[date_index];

            if let Some(strike_data) = strike_slice.get(date) {
                for (type_contract, strikes) in strike_data.iter() {
                    for strike in strikes {
                        let current_opt = contracts_map
                            .get(date)
                            .and_then(|m| m.get(type_contract))
                            .and_then(|m| m.get(strike.into()))
                            .ok_or(format!(
                                "Error accessing {} for date: {}",
                                type_contract, date
                            ))?;

                        let next_date = &dates_slice[date_index + 1];
                        let next_opt = contracts_map
                            .get(next_date)
                            .and_then(|m| m.get(type_contract))
                            .and_then(|m| m.get(strike.into()));

                        if let Some(next_opt) = next_opt {
                            let arb_val = current_opt.mkt - next_opt.mkt;

                            if arb_val > 0.15
                                && current_opt.bid > 0.25
                                && next_opt.bid > 0.25
                                && current_opt.asz > 0.0
                                && next_opt.asz > 0.0
                                && calc_time_difference(date, next_date) == 2
                            {
                                let avg_ask = ((current_opt.asz + next_opt.asz) / 2.0).round();
                                let rank_value =
                                    calc_rank_value(avg_ask, arb_val, &current_date, date);

                                contender_contracts.push(Contender {
                                    arb_val,
                                    avg_ask,
                                    type_spread: "Calendar".to_string(),
                                    exp_date: date.clone(),
                                    rank_value,
                                    contracts: vec![
                                        Contract {
                                            strike: *strike,
                                            mkt_price: current_opt.mkt,
                                            date: date.clone(),
                                            type_contract: type_contract.clone(),
                                        },
                                        Contract {
                                            strike: *strike,
                                            mkt_price: next_opt.mkt,
                                            date: next_date.clone(),
                                            type_contract: type_contract.clone(),
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

    // Function that returns a slice of the top butterfly arbs
    fn get_butterfly_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts = Vec::new();
        let now = Local::now();
        let current_date = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice = self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                for &contract_type in &["C", "P"] {
                    if let Some(contract_strikes) = strike_data.get(contract_type) {
                        if contract_strikes.len() > 2 {
                            for i in 1..(contract_strikes.len() - 1) {
                                let current_strike = &contract_strikes[i];
                                let current_contract = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(current_strike.into()))
                                    .ok_or("Error accessing current contract")?;

                                let left_strike = &contract_strikes[i - 1];
                                let left_contract = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(left_strike.into()))
                                    .ok_or("Error accessing left contract")?;

                                let right_strike = &contract_strikes[i + 1];
                                let right_contract = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(right_strike.into()))
                                    .ok_or("Error accessing right contract")?;

                                let arb_val = (2.0 * current_contract.mkt)
                                    - (left_contract.mkt + right_contract.mkt);

                                if arb_val > 0.15
                                    && left_contract.bid > 0.25
                                    && right_contract.bid > 0.25
                                    && current_contract.bid > 0.25
                                    && left_contract.asz > 0.0
                                    && right_contract.asz > 0.0
                                    && current_contract.asz > 0.0
                                    && (current_strike - left_strike).round() == 5.0
                                    && (right_strike - current_strike).round() == 5.0
                                {
                                    let avg_ask = ((left_contract.asz
                                        + right_contract.asz
                                        + (2.0 * current_contract.asz))
                                        / 4.0)
                                        .round();
                                    let rank_value =
                                        calc_rank_value(avg_ask, arb_val, &current_date, date);

                                    contender_contracts.push(Contender {
                                        arb_val,
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

    // Function that returns a slice of the top boxspread arbs
    fn get_boxspread_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts = Vec::new();
        let now = Local::now();
        let current_date = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice = self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                if let (Some(cs), Some(ps)) = (strike_data.get("C"), strike_data.get("P")) {
                    if cs.len() > 1 && ps.len() > 1 {
                        for i in 0..(cs.len() - 1) {
                            let current_strike_c = &cs[i];
                            let current_c = contracts_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(current_strike_c.into()))
                                .ok_or("Error accessing current_c")?;

                            let current_strike_p = &ps[i];
                            let current_p = contracts_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(current_strike_p.into()))
                                .ok_or("Error accessing current_p")?;

                            let right_strike_c = &cs[i + 1];
                            let right_c = contracts_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(right_strike_c.into()))
                                .ok_or("Error accessing right_c")?;

                            let right_strike_p = &ps[i + 1];
                            let right_p = contracts_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(right_strike_p.into()))
                                .ok_or("Error accessing right_p")?;

                            let arb_val =
                                (current_c.mkt + right_p.mkt) - (current_p.mkt + right_c.mkt) - 5.0;

                            if arb_val > 0.15
                                && current_c.bid > 0.25
                                && current_p.bid > 0.25
                                && right_c.bid > 0.25
                                && right_p.bid > 0.25
                                && current_c.asz > 0.0
                                && current_p.asz > 0.0
                                && right_c.asz > 0.0
                                && right_p.asz > 0.0
                                && (right_strike_c - current_strike_c).round() == 5.0
                                && (right_strike_p - current_strike_p).round() == 5.0
                            {
                                let avg_ask =
                                    ((current_c.asz + right_c.asz + current_p.asz + right_p.asz)
                                        / 4.0)
                                        .round();
                                let rank_value =
                                    calc_rank_value(avg_ask, arb_val, &current_date, date);

                                contender_contracts.push(Contender {
                                    arb_val,
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

    // Function that returns a slice of the top arbs given the number of orders
    fn get_contender_contracts(
        &self,
        option: &str,
        num_orders: i32,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let session_id = self.get_session_id()?;
        let contracts_map = self.get_spx_data(&session_id)?;
        let mut contender_contracts_total = Vec::new();

        match OptionType::from_str(option).ok_or("Invalid option type")? {
            OptionType::Calendar => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map)?);
            }
            OptionType::Butterfly => {
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map)?);
            }
            OptionType::BoxSpread => {
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map)?);
            }
            OptionType::CalendarButterfly => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map)?);
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map)?);
            }
            OptionType::CalendarBoxSpread => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map)?);
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map)?);
            }
            OptionType::ButterflyBoxSpread => {
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map)?);
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map)?);
            }
            OptionType::All => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map)?);
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map)?);
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map)?);
            }
        }

        contender_contracts_total.sort_by(|a, b| b.rank_value.partial_cmp(&a.rank_value).unwrap());

        let num_orders_usize: usize = num_orders as usize; // Direct casting when sure about range.
        if contender_contracts_total.len() > num_orders_usize {
            contender_contracts_total.truncate(num_orders_usize);
        }

        Ok(contender_contracts_total)
    }
}

// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************

fn main() {
    let mut num_orders: i32;
    let mut num_fills: i32;
    let mut port_val: f64;

    let mut active_tick = ActiveTick::new();
    let _ = active_tick.init(&get_username(), &get_password(), &get_api_key(), 5);
    //let mut ibkr = IBKR::new();

    let option = get_option();
    let fill = get_fill_type();
    let mode = get_mode();

    if mode {
        //ibkr.init(get_discount_value(), go_dot_env_variable("DOMAIN"), go_dot_env_variable("PORT"), active_tick.get_dates_slice(), active_tick.get_strike_slice());
        println!("Bot is live !!!");
    }

    loop {
        if is_us_stock_market_open() && is_weekday() || !mode {
            if mode {
                //port_val = ibkr.get_portfolio_value();
                port_val = 100000.0
            } else {
                port_val = 100000.0;
            }

            (num_orders, num_fills) = calc_final_num_orders(&fill, port_val);

            if num_orders > 0 {
                let start_time = Instant::now();

                match active_tick.get_contender_contracts(&option, num_orders) {
                    Ok(contender_contracts) => {
                        if !contender_contracts.is_empty() {
                            if mode {
                                //ibkr.order_contender_contracts(&contender_contracts, num_fills);
                            }
                            for contender in contender_contracts {
                                println!(
                                    "Submitting Order for {} * {} {} @ {:.2}:",
                                    num_fills,
                                    &contender.type_spread,
                                    &contender.exp_date,
                                    contender.arb_value()
                                );

                                for i in 0..contender.contracts.len() {
                                    println!(
                                        "{}",
                                        format_contender_description(&contender, num_fills, i)
                                    );
                                }
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("Error retrieving contender contracts: {}", error);
                        exit(1);
                    }
                }

                // Record the current time after running the program
                let elapsed_time = start_time.elapsed();
                println!("Total time taken: {:?}", elapsed_time);
            } else {
                println!("Not enough equity in account to make a trade !!!");
                break;
            }

            // Sleep to avoid throttling resources
            println!("");
            println!("Sleeping for 30 seconds...");
            sleep(Duration::from_secs(5));
            println!("Awake after 30 seconds!");
            println!("");

            if mode {
                //ibkr.cancel_pending_orders();
            }
        } else {
            println!("Market is closed");
            break;
        }
    }
}

// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************
// ********************************************

fn is_us_stock_market_open() -> bool {
    let market_open_hour = 9;
    let market_open_minute = 30;
    let market_close_hour = 15;
    let market_close_minute = 55;

    let current_time = Utc::now();
    println!("{:?}", current_time);

    let current_hour = current_time.hour();
    let current_minute = current_time.minute();

    if current_hour > market_open_hour && current_hour < market_close_hour {
        return true;
    } else if current_hour == market_open_hour && current_minute >= market_open_minute {
        return true;
    } else if current_hour == market_close_hour && current_minute <= market_close_minute {
        return true;
    }

    // The market is closed.
    false
}

// Function that checks if the current day is a weekday.
fn is_weekday() -> bool {
    let today = Utc::now().weekday();
    today != Weekday::Sat && today != Weekday::Sun
}

// Function that calcs the number of orders and fills for every fill type.
fn calc_final_num_orders(fill: &str, port_val: f64) -> (i32, i32) {
    if port_val / 600.0 < 1.0 {
        return (0, 0);
    }

    match fill {
        "1" => (1, 1),
        "2" => (1, (port_val / 600.0).floor() as i32),
        "3" => ((port_val / 600.0).floor() as i32, 1),
        _ => get_optimal_num_orders(port_val),
    }
}

// Function that gets the ideal number of orders and fills.
fn get_optimal_num_orders(portfolio_value: f64) -> (i32, i32) {
    let num = (portfolio_value / 600.0).sqrt() as i32;
    if num > 9 {
        ((portfolio_value / 600.0 / 9.0).floor() as i32, 9)
    } else {
        (num, num)
    }
}

// Function that returns the number of days between 2 dates.
fn calc_time_difference(current_date: &str, date: &str) -> i64 {
    let current_time = NaiveDate::parse_from_str(current_date, "%y%m%d").unwrap();
    let future_time = NaiveDate::parse_from_str(date, "%y%m%d").unwrap();

    (((current_time - future_time).num_hours() as f64 / 24.0 * -1.0) + 1.0) as i64
}

// Function that returns the rank value for a contract.
fn calc_rank_value(avg_ask: f64, arb_val: f64, current_date: &str, date: &str) -> f64 {
    let difference = calc_time_difference(current_date, date);
    (avg_ask * arb_val) / (difference as f64)
}

// Function that converts dates to the correct format.
fn _convert_date(input_date: &str) -> String {
    let parsed_time = NaiveDate::parse_from_str(input_date, "%y%m%d").unwrap();
    let month_abbreviation = parsed_time.format("%b").to_string().to_uppercase();
    let year_abbreviation = parsed_time.format("%y").to_string();

    format!("{}{}", month_abbreviation, year_abbreviation)
}

// Function that checks if a string exists in a Vec of strings.
fn _string_exists_in_slice(target: &str, slice: &[String]) -> bool {
    slice.contains(&target.to_string())
}

fn format_contender_description(contender: &Contender, num_fills: i32, index: usize) -> String {
    format!(
        "\tLeg {}: {} {} * {:.2}{} {} @ {:.2}",
        index + 1,
        contender.action(index),
        contender.multiplier(num_fills, index),
        contender.contracts[index].strike,
        &contender.contracts[index].type_contract,
        &contender.contracts[index].date,
        contender.contracts[index].mkt_price
    )
}

// Function that gets input and retruns result
fn get_user_input(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

// Function that uses dotenv to load/read the .env file and return the value of the key.
fn get_dotenv_variable(key: &str) -> Result<String, Box<dyn Error>> {
    dotenv()?; // Load the .env file
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(e) => Err(Box::new(e)),
    }
}

// Function that gets username
fn get_username() -> String {
    match get_dotenv_variable("USER_NAME") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter username:"),
    }
}

// Function that gets password
fn get_password() -> String {
    match get_dotenv_variable("PASSWORD") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter password:"),
    }
}

// Function that gets API key
fn get_api_key() -> String {
    match get_dotenv_variable("API_KEY") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter API key:"),
    }
}

// Function that gets option for contracts to look for
fn get_option() -> String {
    match get_dotenv_variable("OPTION") {
        Ok(val) => val,
        Err(_) => {
            let prompt = "\
1 for Calendar
2 for Butterfly
3 for Boxspread
4 for Calendar + Butterfly
5 for Calendar + Boxspread
6 for Butterfly + Boxspread
DEFAULT for Calendar + Butterfly + Boxspread
";
            get_user_input(&format!(
                "{}\nEnter which strategy the bot should use:",
                prompt
            ))
        }
    }
}

// Function that gets fill type
fn get_fill_type() -> String {
    match get_dotenv_variable("FILL_TYPE") {
        Ok(val) => val,
        Err(_) => {
            let prompt = "\
1 for single order, single fill
2 for single order, multiple fills
3 for multiple orders, single fill
DEFAULT for multiple orders, multiple fills
";
            get_user_input(&format!(
                "{}\nEnter which fill type the bot should use:",
                prompt
            ))
        }
    }
}

// Function that gets mode
fn get_mode() -> bool {
    match get_dotenv_variable("TEST_MODE") {
        Ok(val) => val.to_lowercase() != "yes" && val.to_lowercase() != "y",
        Err(_) => {
            let input = get_user_input("Would you like to run the bot in testing mode? (Y / N):");
            input.to_lowercase() != "yes" && input.to_lowercase() != "y"
        }
    }
}

// Function that gets discount value
fn _get_discount_value() -> f64 {
    match get_dotenv_variable("DISCOUNT_VALUE") {
        Ok(val) => match val.parse::<f64>() {
            Ok(val) => {
                if val >= 0.5 && val <= 1.0 {
                    val
                } else {
                    println!("Not a valid Discount Value, setting to 1.0");
                    1.0
                }
            }
            Err(_) => {
                println!("Not a valid Discount Value, setting to 1.0");
                1.0
            }
        },
        Err(_) => {
            let input = get_user_input("Enter a Discount Value between 0.0 and 1.0:");
            match input.parse::<f64>() {
                Ok(val) => {
                    if val >= 0.5 && val <= 1.0 {
                        val
                    } else {
                        println!("Not a valid Discount Value, setting to 1.0");
                        1.0
                    }
                }
                Err(_) => {
                    println!("Not a valid Discount Value, setting to 1.0");
                    1.0
                }
            }
        }
    }
}
