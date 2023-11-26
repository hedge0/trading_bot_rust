use chrono::Local;
use ordered_float::OrderedFloat;
use reqwest::blocking::{Client, Response};
use std::{collections::HashMap, error::Error, process::exit};

use crate::{
    helpers::{get_boxspread_contenders, get_butterfly_contenders, log_error},
    structs::{ChainResponse, Contender, Opt},
};

enum OptionType {
    Butterfly,
    BoxSpread,
    All,
}

impl OptionType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(OptionType::Butterfly),
            "2" => Some(OptionType::BoxSpread),
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
        let session_id: String = String::new();
        Ok(())
    }

    // Function that sends a GET request for SPX data, and then parses the response.
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
            log_error(format!(
                "{}\nBody: {:?}",
                response.status(),
                response.text()?
            ));
            exit(1);
        }

        let chain_results: ChainResponse = response.json()?;
        let mut contracts_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>> =
            HashMap::new();

        for row in chain_results.rows.iter() {
            if row.st == "ok" {
                let (exp_date, type_opt, strike_str) = {
                    let parts: Vec<&str> = row.s.split('_').collect();
                    let code: &str = parts.get(1).ok_or("Invalid format")?;
                    (&code[0..6], &code[6..7], &code[7..(code.len() - 3)])
                };

                let strike: OrderedFloat<f64> = OrderedFloat(
                    strike_str
                        .parse::<f64>()
                        .map_err(|_| "Failed to parse strike")?,
                );
                let bid: f64 = row.data[0].v.parse().map_err(|_| "Failed to parse bid")?;
                let ask: f64 = row.data[1].v.parse().map_err(|_| "Failed to parse ask")?;
                let asz_val: f64 = row.data[2].v.parse().map_err(|_| "Failed to parse asz")?;
                let mkt_val: f64 = ((bid + ask) / 2.0 * 100.0).round() / 100.0;

                let type_map: &mut HashMap<String, HashMap<OrderedFloat<f64>, Opt>> = contracts_map
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

    // Function that returns a slice of the top arbs given the number of orders.
    pub(crate) fn get_contender_contracts(
        &self,
        option: &str,
        num_orders: i32,
    ) -> Result<Vec<Contender>, Box<dyn Error>> {
        let session_id: String = String::new();
        let contracts_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>> =
            self.get_spx_data(&session_id)?;
        let mut contender_contracts_total: Vec<Contender> = Vec::new();

        let dates_slice: &Vec<String> =
            self.dates_slice.as_ref().ok_or("dates slice is not set")?;
        let strike_slice: &HashMap<String, HashMap<String, Vec<f64>>> = self
            .strike_slice
            .as_ref()
            .ok_or("strike slice is not set")?;

        match OptionType::from_str(option).ok_or("Invalid option type")? {
            OptionType::Butterfly => {
                contender_contracts_total.extend(get_butterfly_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                )?);
            }
            OptionType::BoxSpread => {
                contender_contracts_total.extend(get_boxspread_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                )?);
            }
            OptionType::All => {
                contender_contracts_total.extend(get_butterfly_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                )?);
                contender_contracts_total.extend(get_boxspread_contenders(
                    &contracts_map,
                    dates_slice,
                    strike_slice,
                )?);
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
