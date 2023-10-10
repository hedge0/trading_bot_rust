use chrono::{Datelike, NaiveDate, Timelike, Utc, Weekday};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::io;

pub(crate) fn is_us_stock_market_open() -> bool {
    let market_open_hour: u32 = 9;
    let market_open_minute: u32 = 30;
    let market_close_hour: u32 = 15;
    let market_close_minute: u32 = 55;

    let current_time: chrono::DateTime<Utc> = Utc::now();
    println!("{:?}", current_time);

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

    (((current_time - future_time).num_hours() as f64 / 24.0 * -1.0) + 1.0) as i64
}

// Function that returns the rank value for a contract.
pub(crate) fn calc_rank_value(avg_ask: f64, arb_val: f64, current_date: &str, date: &str) -> f64 {
    let difference: i64 = calc_time_difference(current_date, date);
    (avg_ask * arb_val) / (difference as f64)
}

// Function that converts dates to the correct format.
pub(crate) fn convert_date(input_date: &str) -> String {
    let parsed_time: NaiveDate = NaiveDate::parse_from_str(input_date, "%y%m%d").unwrap();
    let month_abbreviation: String = parsed_time.format("%b").to_string().to_uppercase();
    let year_abbreviation: String = parsed_time.format("%y").to_string();

    format!("{}{}", month_abbreviation, year_abbreviation)
}

// Function that checks if a string exists in a Vec of strings.
fn _string_exists_in_slice(target: &str, slice: &[String]) -> bool {
    slice.contains(&target.to_string())
}

// Function that gets input and retruns result
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

// Function that gets username
pub(crate) fn get_username() -> String {
    match get_dotenv_variable("USER_NAME") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter username:"),
    }
}

// Function that gets password
pub(crate) fn get_password() -> String {
    match get_dotenv_variable("PASSWORD") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter password:"),
    }
}

// Function that gets API key
pub(crate) fn get_api_key() -> String {
    match get_dotenv_variable("API_KEY") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter API key:"),
    }
}

// Function that gets option for contracts to look for
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

// Function that gets fill type
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

// Function that gets mode
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

// Function that gets discount value
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
