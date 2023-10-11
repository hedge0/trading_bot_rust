use chrono::{Datelike, Local};
use ordered_float::OrderedFloat;
use reqwest::blocking::{Client, Response};
use simd_json;
use std::error::Error;
use std::process::exit;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};

use crate::{
    helpers::{calc_rank_value, calc_time_difference},
    structs::{AuthResponse, ChainResponse, Contender, Contract, Opt},
};

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

pub(crate) struct ActiveTick {
    username: Option<String>,
    password: Option<String>,
    apikey: Option<String>,
    num_days: Option<std::time::Duration>,
    client: Option<Client>,
    dates_slice: Option<Vec<String>>,
    strike_slice: Option<HashMap<String, HashMap<String, Vec<f64>>>>,
}

impl ActiveTick {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn init(
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
        let session_id: String = self.get_session_id()?;
        let (dates, strikes) = self.get_spx_dates_and_strikes(&session_id)?;
        self.dates_slice = dates;
        self.strike_slice = strikes;
        Ok(())
    }

    // Function that returns datesSlice
    pub(crate) fn get_dates_slice(&self) -> Vec<String> {
        match &self.dates_slice {
            Some(dates) => dates.clone(),
            None => {
                println!("Error: dates_slice is None.");
                exit(1);
            }
        }
    }

    // Function that returns strikeSlice
    pub(crate) fn get_strike_slice(&self) -> HashMap<String, HashMap<String, Vec<f64>>> {
        match &self.strike_slice {
            Some(strikes) => strikes.clone(),
            None => {
                println!("Error: strike_slice is None.");
                exit(1);
            }
        }
    }

    // Function that checks if the user is authorized to use the Activetick API, and returns a sessionID if ok
    fn get_session_id(&self) -> Result<String, Box<dyn Error>> {
        let auth_url: &str = "https://api.activetick.com/authorize.json";

        let params: [(&str, &String); 3] = [
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

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(auth_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
            exit(1);
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

        eprintln!("Error: Failed to get session ID");
        exit(1);
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
        let chain_url: &str = "https://api.activetick.com/chain.json";
        let current_time: chrono::DateTime<Local> = chrono::Local::now();
        let future_time: chrono::DateTime<Local> = current_time + self.num_days.unwrap_or_default(); // handle None case
        let formatted_time: String = current_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let formatted_future_time: String = future_time.format("%Y-%m-%dT%H:%M:%S").to_string();

        let params: [(&str, &str); 7] = [
            ("sessionid", session_id),
            ("key", "SPXW_S U"),
            ("chaintype", "equity_options"),
            ("columns", "b,a,asz"),
            ("begin_maturity_time", &formatted_time),
            ("end_maturity_time", &formatted_future_time),
            ("ignore_empty", "false"),
        ];

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(chain_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
            exit(1);
        }

        let chain_results: ChainResponse = response.json()?;
        if chain_results.rows.is_empty() {
            eprintln!("Error: SPX rows data is empty");
            exit(1);
        }
        let mut dates_slice: Vec<String> = Vec::new();
        let mut strike_slice: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();

        for row in chain_results.rows.iter() {
            if row.st == "ok" {
                let parts: Vec<&str> = row.s.split('_').collect();
                let code: &str = parts[1];
                let exp_date: &str = &code[0..6];
                let type_opt: &str = &code[6..7];
                let strike_str: &str = &code[7..(code.len() - 3)];
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

        return Ok((Some(dates_slice), Some(strike_slice)));
    }

    // Function that sends a GET request for SPX data, and then parses the response
    fn get_spx_data(
        &self,
        session_id: &str,
    ) -> Result<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>, Box<dyn Error>>
    {
        let chain_url: &str = "https://api.activetick.com/chain.json";
        let current_time: chrono::DateTime<Local> = chrono::Local::now();
        let future_time: chrono::DateTime<Local> =
            current_time + self.num_days.ok_or("num_days is not set")?;

        let params: [(&str, &str); 7] = [
            ("sessionid", session_id),
            ("key", "SPXW_S U"),
            ("chaintype", "equity_options"),
            ("columns", "b,a,asz"),
            (
                "begin_maturity_time",
                &current_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ),
            (
                "end_maturity_time",
                &future_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ),
            ("ignore_empty", "false"),
        ];

        let response: Response = self
            .client
            .as_ref()
            .ok_or("Client is not initialized")?
            .get(chain_url)
            .header("Connection", "keep-alive")
            .query(&params)
            .send()?;

        if !response.status().is_success() {
            eprintln!("Error: {}\nBody: {:?}", response.status(), response.text()?);
            exit(1);
        }

        let start_time: Instant = Instant::now();

        // Read the response text first
        let response_text = response.text()?;

        let mut buf = response_text.into_bytes();
        buf.resize(buf.len() + simd_json::buffer::extra_capacity(), 0);

        // Parse to a simd-json Value
        let parsed_value: simd_json::BorrowedValue = simd_json::to_borrowed_value(&mut &buf)?;

        // Convert to a serde_json::Value
        let json_value: serde_json::Value = serde_json::from_str(&parsed_value.to_string())?;

        // Deserialize to your struct
        let chain_results: ChainResponse = serde_json::from_value(json_value)?;

        let elapsed_time: Duration = start_time.elapsed();
        println!("Total time taken: {:?}", elapsed_time);

        let mut contracts_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>> =
            HashMap::new();

        for row in chain_results.rows.iter() {
            if row.st == "ok" {
                let (exp_date, type_opt, strike_str) = {
                    let parts: Vec<&str> = row.s.split('_').collect();
                    let code: &str = parts.get(1).ok_or("Invalid format")?;
                    (&code[0..6], &code[6..7], &code[7..(code.len() - 3)])
                };

                let strike = OrderedFloat(
                    strike_str
                        .parse::<f64>()
                        .map_err(|_| "Failed to parse strike")?,
                );
                let bid: f64 = row.data[0].v.parse().map_err(|_| "Failed to parse bid")?;
                let ask: f64 = row.data[1].v.parse().map_err(|_| "Failed to parse ask")?;
                let asz_val: f64 = row.data[2].v.parse().map_err(|_| "Failed to parse asz")?;
                let mkt_val: f64 = ((bid + ask) / 2.0 * 100.0).round() / 100.0;

                let type_map = contracts_map
                    .entry(exp_date.to_string())
                    .or_insert_with(|| {
                        let mut m: HashMap<String, HashMap<OrderedFloat<f64>, Opt>> =
                            HashMap::new();
                        m.insert("C".to_string(), HashMap::new());
                        m.insert("P".to_string(), HashMap::new());
                        m
                    });

                type_map.entry(type_opt.to_string()).or_default().insert(
                    strike,
                    Opt {
                        asz: asz_val,
                        mkt: mkt_val,
                        bid,
                    },
                );
            }
        }

        return Ok(contracts_map);
    }

    // Function that returns a slice of the top calendar arbs
    fn get_calendar_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice: &Vec<String> =
            self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice: &HashMap<String, HashMap<String, Vec<f64>>> = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date_index in 0..(dates_slice.len() - 1) {
            let date: &String = &dates_slice[date_index];

            if let Some(strike_data) = strike_slice.get(date) {
                for (type_contract, strikes) in strike_data.iter() {
                    for strike in strikes {
                        let current_opt: &Opt = contracts_map
                            .get(date)
                            .and_then(|m| m.get(type_contract))
                            .and_then(|m| m.get(strike.into()))
                            .ok_or(format!(
                                "Error accessing {} for date: {}",
                                type_contract, date
                            ))?;

                        let next_date: &String = &dates_slice[date_index + 1];
                        let next_opt: Option<&Opt> = contracts_map
                            .get(next_date)
                            .and_then(|m| m.get(type_contract))
                            .and_then(|m| m.get(strike.into()));

                        if let Some(next_opt) = next_opt {
                            let arb_val: f64 = current_opt.mkt - next_opt.mkt;

                            if arb_val > 0.15
                                && current_opt.bid > 0.25
                                && next_opt.bid > 0.25
                                && current_opt.asz > 0.0
                                && next_opt.asz > 0.0
                                && calc_time_difference(date, next_date) == 2
                            {
                                let avg_ask: f64 = ((current_opt.asz + next_opt.asz) / 2.0).round();
                                let rank_value: f64 =
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

        return Ok(contender_contracts);
    }

    // Function that returns a slice of the top butterfly arbs
    fn get_butterfly_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice: &Vec<String> =
            self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice: &HashMap<String, HashMap<String, Vec<f64>>> = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                for &contract_type in &["C", "P"] {
                    if let Some(contract_strikes) = strike_data.get(contract_type) {
                        if contract_strikes.len() > 2 {
                            for i in 1..(contract_strikes.len() - 1) {
                                let current_strike: &f64 = &contract_strikes[i];
                                let current_contract: &Opt = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(current_strike.into()))
                                    .ok_or("Error accessing current contract")?;

                                let left_strike: &f64 = &contract_strikes[i - 1];
                                let left_contract: &Opt = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(left_strike.into()))
                                    .ok_or("Error accessing left contract")?;

                                let right_strike: &f64 = &contract_strikes[i + 1];
                                let right_contract: &Opt = contracts_map
                                    .get(date)
                                    .and_then(|ct| ct.get(contract_type))
                                    .and_then(|ct| ct.get(right_strike.into()))
                                    .ok_or("Error accessing right contract")?;

                                let arb_val: f64 = (2.0 * current_contract.mkt)
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
                                    let avg_ask: f64 = ((left_contract.asz
                                        + right_contract.asz
                                        + (2.0 * current_contract.asz))
                                        / 4.0)
                                        .round();
                                    let rank_value: f64 =
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

        return Ok(contender_contracts);
    }

    // Function that returns a slice of the top boxspread arbs
    fn get_boxspread_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let mut contender_contracts: Vec<Contender> = Vec::new();
        let now: chrono::DateTime<Local> = Local::now();
        let current_date: String =
            format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

        let dates_slice: &Vec<String> =
            self.dates_slice.as_ref().ok_or("dates_slice is not set")?;
        let strike_slice: &HashMap<String, HashMap<String, Vec<f64>>> = self
            .strike_slice
            .as_ref()
            .ok_or("strike_slice is not set")?;

        for date in dates_slice {
            if let Some(strike_data) = strike_slice.get(date) {
                if let (Some(cs), Some(ps)) = (strike_data.get("C"), strike_data.get("P")) {
                    if cs.len() > 1 && ps.len() > 1 {
                        for i in 0..(cs.len() - 1) {
                            let current_strike_c: &f64 = &cs[i];
                            let current_c: &Opt = contracts_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(current_strike_c.into()))
                                .ok_or("Error accessing current_c")?;

                            let current_strike_p: &f64 = &ps[i];
                            let current_p: &Opt = contracts_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(current_strike_p.into()))
                                .ok_or("Error accessing current_p")?;

                            let right_strike_c: &f64 = &cs[i + 1];
                            let right_c: &Opt = contracts_map
                                .get(date)
                                .and_then(|c| c.get("C"))
                                .and_then(|c| c.get(right_strike_c.into()))
                                .ok_or("Error accessing right_c")?;

                            let right_strike_p: &f64 = &ps[i + 1];
                            let right_p: &Opt = contracts_map
                                .get(date)
                                .and_then(|p| p.get("P"))
                                .and_then(|p| p.get(right_strike_p.into()))
                                .ok_or("Error accessing right_p")?;

                            let arb_val: f64 =
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
                                let avg_ask: f64 =
                                    ((current_c.asz + right_c.asz + current_p.asz + right_p.asz)
                                        / 4.0)
                                        .round();
                                let rank_value: f64 =
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

        return Ok(contender_contracts);
    }

    // Function that returns a slice of the top arbs given the number of orders
    pub(crate) fn get_contender_contracts(
        &self,
        option: &str,
        num_orders: i32,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let session_id: String = self.get_session_id()?;
        let contracts_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>> =
            self.get_spx_data(&session_id)?;
        let mut contender_contracts_total: Vec<Contender> = Vec::new();

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

        return Ok(contender_contracts_total);
    }
}
