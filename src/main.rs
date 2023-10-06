use chrono::{Datelike, Timelike, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::time::SystemTime;

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

#[derive(Serialize, Deserialize)]
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

struct ActiveTick {
    username: String,
    password: String,
    apikey: String,
    num_days: std::time::Duration,
    dates_slice: Vec<String>,
    strike_slice: HashMap<String, HashMap<String, Vec<f64>>>,
}

fn main() {
    // Call the is_us_stock_market_open() function from the helpers module
    if is_us_stock_market_open() {
        println!("The US stock market is open.");
    } else {
        println!("The US stock market is closed.");
    }
}

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
    today != chrono::Weekday::Sat && today != chrono::Weekday::Sun
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

// Function that uses dotenv to load/read the .env file and return the value of the key.
fn get_dotenv_variable(key: &str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(_) => {
            println!("Error: Environment variable not found");
            std::process::exit(1);
        }
    }
}

// Function that returns the number of days between 2 dates.
fn calc_time_difference(current_date: &str, date: &str) -> f64 {
    let current_time = chrono::NaiveDate::parse_from_str(current_date, "%y%m%d").unwrap();
    let future_time = chrono::NaiveDate::parse_from_str(date, "%y%m%d").unwrap();

    ((current_time - future_time).num_hours() as f64 / 24.0 * -1.0) + 1.0
}

// Function that returns the rank value for a contract.
fn calc_rank_value(avg_ask: f64, arb_val: f64, current_date: &str, date: &str) -> f64 {
    let difference = calc_time_difference(current_date, date);
    (avg_ask * arb_val) / difference
}

// Function that converts dates to the correct format.
fn convert_date(input_date: &str) -> String {
    let parsed_time = chrono::NaiveDate::parse_from_str(input_date, "%y%m%d").unwrap();
    let month_abbreviation = parsed_time.format("%b").to_string().to_uppercase();
    let year_abbreviation = parsed_time.format("%y").to_string();

    format!("{}{}", month_abbreviation, year_abbreviation)
}

// Function that checks if a string exists in a Vec of strings.
fn string_exists_in_slice(target: &str, slice: &[String]) -> bool {
    slice.contains(&target.to_string())
}

fn get_user_input(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

// Function that gets username
fn get_username() -> String {
    let username = get_dotenv_variable("USER_NAME");
    if !username.is_empty() {
        username
    } else {
        get_user_input("Enter username:")
    }
}

// Function that gets password
fn get_password() -> String {
    let password = get_dotenv_variable("PASSWORD");
    if !password.is_empty() {
        password
    } else {
        get_user_input("Enter password:")
    }
}

// Function that gets API key
fn get_api_key() -> String {
    let api_key = get_dotenv_variable("API_KEY");
    if !api_key.is_empty() {
        api_key
    } else {
        get_user_input("Enter API key:")
    }
}

// Function that gets option for contracts to look for
fn get_option() -> String {
    let option = get_dotenv_variable("OPTION");
    if !option.is_empty() {
        option
    } else {
        let prompt = "\
1 for Calendar
2 for Butterfly
3 for Boxspread
4 for Calendar + Butterfly
5 for Calendar + Boxspread
6 for Butterfly + Boxspread
DEFAULT for Calendar + Butterfly + Boxspread";
        get_user_input(&format!(
            "Enter which strategy the bot should use:\n{}",
            prompt
        ))
    }
}

// Function that gets fill type
fn get_fill_type() -> String {
    let fill = get_dotenv_variable("FILL_TYPE");
    if !fill.is_empty() {
        fill
    } else {
        let prompt = "\
1 for single order, single fill
2 for single order, multiple fills
3 for multiple orders, single fill
DEFAULT for multiple orders, multiple fills";
        get_user_input(&format!(
            "Enter which fill type the bot should use:\n{}",
            prompt
        ))
    }
}

// Function that gets mode
fn get_mode() -> bool {
    let mode = get_dotenv_variable("TEST_MODE");
    if !mode.is_empty() {
        mode.to_lowercase() != "yes" && mode.to_lowercase() != "y"
    } else {
        let input = get_user_input("Would you like to run the bot in testing mode? (Y / N):");
        input.to_lowercase() != "yes" && input.to_lowercase() != "y"
    }
}

// Function that gets discount value
fn get_discount_value() -> f64 {
    let discount_value = get_dotenv_variable("DISCOUNT_VALUE");
    if !discount_value.is_empty() {
        match discount_value.parse::<f64>() {
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
    } else {
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
