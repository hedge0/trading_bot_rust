use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::America::New_York;
use dotenv::dotenv;
use std::{env, error::Error, io::stdin};

// Function that gets input and returns result.
fn get_user_input(prompt: &str) -> String {
    let mut input: String = String::new();
    println!("{}", prompt);
    stdin().read_line(&mut input).expect("Failed to read line");
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

// Function that gets option for contracts to look for.
pub(crate) fn get_option() -> String {
    match get_dotenv_variable("OPTION") {
        Ok(val) => val,
        Err(_) => {
            let prompt: &str = "\
1 for Butterfly
2 for Boxspread
DEFAULT for Butterfly + Boxspread
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
    // Convert the current UTC time to New York time
    let ny_time: DateTime<chrono_tz::Tz> = current_time.with_timezone(&New_York);

    // Check if it's a weekday (NYSE and NASDAQ are closed on weekends)
    if ny_time.weekday() == Weekday::Sat || ny_time.weekday() == Weekday::Sun {
        return false;
    }

    // Define market opening and closing hours in New York time using with_ymd_and_hms()
    let market_open_result: chrono::LocalResult<DateTime<chrono_tz::Tz>> =
        New_York.with_ymd_and_hms(ny_time.year(), ny_time.month(), ny_time.day(), 9, 30, 0);
    let market_close_result: chrono::LocalResult<DateTime<chrono_tz::Tz>> =
        New_York.with_ymd_and_hms(ny_time.year(), ny_time.month(), ny_time.day(), 15, 0, 0);

    let market_open: DateTime<chrono_tz::Tz> = match market_open_result.single() {
        Some(time) => time,
        None => return false,
    };

    let market_close: DateTime<chrono_tz::Tz> = match market_close_result.single() {
        Some(time) => time,
        None => return false,
    };

    // Check if the current time is within market hours
    ny_time >= market_open && ny_time <= market_close
}

// Function that calcs the number of orders and fills for every fill type.
pub(crate) fn calc_final_num_orders(fill: &str, port_val: f64) -> (i32, i32) {
    let num_times: i32 = (port_val / 600.0).floor() as i32;

    if num_times < 1 {
        return (0, 0);
    }

    match fill {
        "1" => (1, 1),
        "2" => (1, if num_times > 9 { 9 } else { num_times }),
        "3" => (num_times, 1),
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
