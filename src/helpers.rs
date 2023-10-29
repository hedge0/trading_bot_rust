use chrono::{Datelike, Local, NaiveDate, Timelike, Utc, Weekday};
use dotenv::dotenv;
use ordered_float::OrderedFloat;
use std::{collections::HashMap, env, error::Error, io};

use crate::structs::{Contender, Contract, Opt, OrderBody};

// Function that gets input and returns result.
fn get_user_input(prompt: &str) -> String {
    let mut input: String = String::new();
    println!("{}", prompt);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

// Function that uses dotenv to load/read the .env file and return the value of the key.
pub(crate) fn get_dotenv_variable(key: &str) -> Result<String, Box<dyn Error>> {
    dotenv()?; // Load the .env file.
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(e) => Err(Box::new(e)),
    }
}

// Function that gets username.
pub(crate) fn get_username() -> String {
    match get_dotenv_variable("USER_NAME") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter username:"),
    }
}

// Function that gets password.
pub(crate) fn get_password() -> String {
    match get_dotenv_variable("PASSWORD") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter password:"),
    }
}

// Function that gets API key.
pub(crate) fn get_api_key() -> String {
    match get_dotenv_variable("API_KEY") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter API key:"),
    }
}

// Function that gets option for contracts to look for.
pub(crate) fn get_option() -> String {
    match get_dotenv_variable("OPTION") {
        Ok(val) => val,
        Err(_) => {
            let prompt: &str = "\
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

// Function that gets fill type.
pub(crate) fn get_fill_type() -> String {
    match get_dotenv_variable("FILL_TYPE") {
        Ok(val) => val,
        Err(_) => {
            let prompt: &str = "\
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

// Function that gets mode.
pub(crate) fn get_mode() -> bool {
    match get_dotenv_variable("TEST_MODE") {
        Ok(val) => val.to_lowercase() != "yes" && val.to_lowercase() != "y",
        Err(_) => {
            let input: String =
                get_user_input("Would you like to run the bot in testing mode? (Y / N):");
            input.to_lowercase() != "yes" && input.to_lowercase() != "y"
        }
    }
}

// Function that gets seconds to wait before cancelling and trying again.
pub(crate) fn get_seconds_to_sleep() -> u64 {
    match get_dotenv_variable("SECONDS_TO_SLEEP") {
        Ok(val) => {
            match val.parse::<u64>() {
                Ok(parsed_val) => {
                    if parsed_val > 5 {
                        parsed_val
                    } else {
                        println!("Not a valid positive integer greater than or equal to 5, setting to 60");
                        60
                    }
                }
                Err(_) => {
                    println!("Not a valid integer, setting to 60");
                    60
                }
            }
        }
        Err(_) => {
            let input: String = get_user_input("Enter number of seconds to sleep:");
            match input.parse::<u64>() {
                Ok(parsed_input) => {
                    if parsed_input > 5 {
                        parsed_input
                    } else {
                        println!("Not a valid positive integer greater than or equal to 5, setting to 60");
                        60
                    }
                }
                Err(_) => {
                    println!("Not a valid integer, setting to 60");
                    60
                }
            }
        }
    }
}

// Function that gets discount value.
pub(crate) fn get_discount_value() -> f64 {
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
            let input: String = get_user_input("Enter a Discount Value between 0.0 and 1.0:");
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

// Function that checks if the stock market is currently open.
pub(crate) fn is_us_stock_market_open(current_time: chrono::DateTime<Utc>) -> bool {
    let market_open_hour: u32 = 9;
    let market_open_minute: u32 = 30;
    let market_close_hour: u32 = 15;
    let market_close_minute: u32 = 15;

    let current_hour: u32 = current_time.hour();
    let current_minute: u32 = current_time.minute();

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
pub(crate) fn is_weekday() -> bool {
    let today: Weekday = Utc::now().weekday();
    today != Weekday::Sat && today != Weekday::Sun
}

// Function that calcs the number of orders and fills for every fill type.
pub(crate) fn calc_final_num_orders(fill: &str, port_val: f64) -> (i32, i32) {
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
    let num: i32 = (portfolio_value / 600.0).sqrt() as i32;
    if num > 9 {
        ((portfolio_value / 600.0 / 9.0).floor() as i32, 9)
    } else {
        (num, num)
    }
}

// Function that returns the number of days between 2 dates.
pub(crate) fn calc_time_difference(current_date: &str, date: &str) -> i64 {
    let current_time: NaiveDate = NaiveDate::parse_from_str(current_date, "%y%m%d").unwrap();
    let future_time: NaiveDate = NaiveDate::parse_from_str(date, "%y%m%d").unwrap();

    ((current_time - future_time).num_hours() as f64 / 24.0 * -1.0) as i64
}

// Function that returns the rank value for a contract.
pub(crate) fn calc_rank_value(avg_ask: f64, arb_val: f64, current_date: &str, date: &str) -> f64 {
    let difference: i64 = calc_time_difference(current_date, date) + 1;
    (avg_ask * arb_val) / (difference as f64)
}

// Function that converts dates to the correct format.
pub(crate) fn convert_date(input_date: &str) -> String {
    let parsed_time: NaiveDate = NaiveDate::parse_from_str(input_date, "%y%m%d").unwrap();
    let month_abbreviation: String = parsed_time.format("%b").to_string().to_uppercase();
    let year_abbreviation: String = parsed_time.format("%y").to_string();

    format!("{}{}", month_abbreviation, year_abbreviation)
}

// Function that inits the conids map to have a proper structure for later.
pub(crate) fn generate_conids_structure(
    dates_slice: &[String],
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
) -> HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> {
    let mut conids_map: HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>> =
        HashMap::new();

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

    conids_map
}

// Function that inits the months slice with the correct string formats for later when called into the api endpoint.
pub(crate) fn generate_months_slice(dates_slice: &[String]) -> Vec<String> {
    let mut months_slice: Vec<String> = Vec::new();

    for date in dates_slice {
        let formatted_date: String = convert_date(date);
        if !months_slice.contains(&formatted_date) {
            months_slice.push(formatted_date);
        }
    }

    months_slice
}

// Function that builds calendar order body.
pub(crate) fn build_calendar_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-1,{}/1",
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()]
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: -1.0 * ((contract.arb_val * discount_value.unwrap() * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

// Function that builds calendar order body.
pub(crate) fn build_butterfly_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-2,{}/1,{}/1",
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()]
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[2].date.as_str()]
                [contract.contracts[2].type_contract.as_str()]
                [(&contract.contracts[2].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: -1.0 * ((contract.arb_val * discount_value.unwrap() * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

// Function that builds calendar order body.
pub(crate) fn build_boxspread_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-1,{}/1,{}/1,{}/-1",
            conids_map.as_ref().unwrap()[contract.contracts[3].date.as_str()]
                [contract.contracts[3].type_contract.as_str()]
                [(&contract.contracts[3].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[2].date.as_str()]
                [contract.contracts[2].type_contract.as_str()]
                [(&contract.contracts[2].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()]
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: -1.0 * ((contract.arb_val * discount_value.unwrap() * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

// Function that builds request data for json body to submit an order.
pub(crate) fn build_request_data(
    contender_contracts: &Vec<Contender>,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> Vec<OrderBody> {
    let mut request_data: Vec<OrderBody> = Vec::new();

    for contract in contender_contracts {
        match contract.type_spread.as_str() {
            "Calendar" => {
                request_data.push(build_calendar_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            "Butterfly" => {
                request_data.push(build_butterfly_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            "Boxspread" => {
                request_data.push(build_boxspread_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            _ => {}
        }
    }

    request_data
}

// Function that returns a slice of the top calendar arbs.
pub(crate) fn get_calendar_contenders(
    contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    dates_slice: &Vec<String>,
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
) -> Result<Vec<Contender>, Box<dyn Error>> {
    let mut contender_contracts: Vec<Contender> = Vec::new();
    let now: chrono::DateTime<Local> = Local::now();
    let current_date: String = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

    for date_index in 0..(dates_slice.len() - 1) {
        let date: &String = &dates_slice[date_index];

        if let Some(strike_data) = strike_slice.get(date) {
            for (type_contract, strikes) in strike_data.iter() {
                for strike in strikes {
                    let current_opt: &Opt = contracts_map
                        .get(date)
                        .and_then(|m| m.get(type_contract))
                        .and_then(|m| m.get(&OrderedFloat(*strike)))
                        .ok_or("Error accessing current contract")?;

                    let next_date: &String = &dates_slice[date_index + 1];
                    let next_opt: Option<&Opt> = contracts_map
                        .get(next_date)
                        .and_then(|m| m.get(type_contract))
                        .and_then(|m| m.get(&OrderedFloat(*strike)));

                    if let Some(next_opt) = next_opt {
                        let arb_val: f64 = current_opt.mkt - next_opt.mkt;

                        if arb_val > 0.15
                            && current_opt.bid > 1.0
                            && next_opt.bid > 1.0
                            && current_opt.asz > 0.0
                            && next_opt.asz > 0.0
                            && calc_time_difference(date, next_date) == 1
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

// Function that returns a slice of the top butterfly arbs.
pub(crate) fn get_butterfly_contenders(
    contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    dates_slice: &Vec<String>,
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
) -> Result<Vec<Contender>, Box<dyn Error>> {
    let mut contender_contracts: Vec<Contender> = Vec::new();
    let now: chrono::DateTime<Local> = Local::now();
    let current_date: String = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

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
                                && left_contract.bid > 1.0
                                && right_contract.bid > 1.0
                                && current_contract.bid > 1.0
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

// Function that returns a slice of the top boxspread arbs.
pub(crate) fn get_boxspread_contenders(
    contracts_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, Opt>>>,
    dates_slice: &Vec<String>,
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
) -> Result<Vec<Contender>, Box<dyn Error>> {
    let mut contender_contracts: Vec<Contender> = Vec::new();
    let now: chrono::DateTime<Local> = Local::now();
    let current_date: String = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

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
                            .ok_or("Error accessing current call contract")?;

                        let current_strike_p: &f64 = &ps[i];
                        let current_p: &Opt = contracts_map
                            .get(date)
                            .and_then(|p| p.get("P"))
                            .and_then(|p| p.get(current_strike_p.into()))
                            .ok_or("Error accessing current put contract")?;

                        let right_strike_c: &f64 = &cs[i + 1];
                        let right_c: &Opt = contracts_map
                            .get(date)
                            .and_then(|c| c.get("C"))
                            .and_then(|c| c.get(right_strike_c.into()))
                            .ok_or("Error accessing right call contract")?;

                        let right_strike_p: &f64 = &ps[i + 1];
                        let right_p: &Opt = contracts_map
                            .get(date)
                            .and_then(|p| p.get("P"))
                            .and_then(|p| p.get(right_strike_p.into()))
                            .ok_or("Error accessing right put contract")?;

                        let arb_val: f64 =
                            (current_c.mkt + right_p.mkt) - (current_p.mkt + right_c.mkt);

                        if arb_val > 0.15
                            && current_c.bid > 1.0
                            && current_p.bid > 1.0
                            && right_c.bid > 1.0
                            && right_p.bid > 1.0
                            && current_c.asz > 0.0
                            && current_p.asz > 0.0
                            && right_c.asz > 0.0
                            && right_p.asz > 0.0
                            && (right_strike_c - current_strike_c).round() == 5.0
                            && (right_strike_p - current_strike_p).round() == 5.0
                        {
                            let avg_ask: f64 =
                                ((current_c.asz + right_c.asz + current_p.asz + right_p.asz) / 4.0)
                                    .round();
                            let rank_value: f64 =
                                calc_rank_value(avg_ask, arb_val, &current_date, date);

                            contender_contracts.push(Contender {
                                arb_val: (arb_val * 100.0).round() / 100.0,
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
