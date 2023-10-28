use chrono::{Datelike, NaiveDate, Timelike, Utc, Weekday};
use dotenv::dotenv;
use ordered_float::OrderedFloat;
use std::{collections::HashMap, env, error::Error, io};

use crate::structs::{Contender, OrderBody};

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
            "28812380;;;{}/-1,{}",
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
        price: -1.0 * (contract.arb_val * discount_value.unwrap()).round(),
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
            "28812380;;;{}/-2,{},{}",
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
        price: -1.0 * (contract.arb_val * discount_value.unwrap()).round(),
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
            "28812380;;;{}/-1,{},{},{}",
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
        price: -1.0 * ((contract.arb_val * discount_value.unwrap()) + 5.0).round(),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

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
    dotenv()?; // Load the .env file
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
