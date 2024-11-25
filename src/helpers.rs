use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::America::New_York;
use dotenv::dotenv;
use std::{env, error::Error, io::stdin};

/// Prompts the user with the provided message and returns the input as a trimmed `String`.
///
/// # Arguments
///
/// * `prompt` - A message to display to the user before capturing input.
///
/// # Returns
///
/// A `String` containing the user's input, trimmed of leading/trailing whitespace.
///
/// # Example
///
/// ```
/// let user_input = get_user_input("Enter a value:");
/// println!("You entered: {}", user_input);
/// ```
fn get_user_input(prompt: &str) -> String {
    let mut input: String = String::new();
    println!("{}", prompt);
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

/// Loads a variable from the `.env` file based on the provided key.
///
/// # Arguments
///
/// * `key` - The key corresponding to the desired environment variable.
///
/// # Returns
///
/// * `Ok(String)` containing the value of the environment variable if found.
/// * `Err(Box<dyn Error>)` if the key is not found or any error occurs.
///
/// # Example
///
/// ```
/// let ticker = get_dotenv_variable("TICKER").unwrap();
/// ```
pub(crate) fn get_dotenv_variable(key: &str) -> Result<String, Box<dyn Error>> {
    dotenv()?; // Load the .env file.
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(e) => Err(Box::new(e)),
    }
}

/// Gets the selected option strategy based on user input or the `.env` file.
///
/// # Returns
///
/// A `String` representing the chosen strategy:
/// * `"1"` for Calendar.
/// * `"2"` for Butterfly.
/// * `"3"` for Boxspread.
/// * `"DEFAULT"` for all strategies.
///
/// # Example
///
/// ```
/// let strategy = get_option();
/// println!("Selected strategy: {}", strategy);
/// ```
pub(crate) fn get_option() -> String {
    match get_dotenv_variable("OPTION") {
        Ok(val) => val,
        Err(_) => {
            let prompt: &str = "\
1 for Calendar
2 for Butterfly
3 for Boxspread
DEFAULT for Calendar + Butterfly + Boxspread
";
            get_user_input(&format! {
            "{}\nEnter which strategy the bot should use:", prompt})
        }
    }
}

/// Gets the selected fill type based on user input or the `.env` file.
///
/// # Returns
///
/// A `String` representing the fill type:
/// * `"1"` for single order, single fill.
/// * `"2"` for single order, multiple fills.
/// * `"3"` for multiple orders, single fill.
/// * `"DEFAULT"` for multiple orders, multiple fills.
///
/// # Example
///
/// ```
/// let fill_type = get_fill_type();
/// println!("Selected fill type: {}", fill_type);
/// ```
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
            get_user_input(&format! {
            "{}\nEnter which fill type the bot should use:", prompt})
        }
    }
}

/// Gets the ticker based on user input or the `.env` file.
///
/// # Returns
///
/// A `String` representing the selected ticker.
///
/// # Example
///
/// ```
/// let ticker = get_ticker();
/// println!("Selected ticker: {}", ticker);
/// ```
pub(crate) fn get_ticker() -> String {
    match get_dotenv_variable("TICKER") {
        Ok(val) => val,
        Err(_) => get_user_input("Enter which ticker the bot should use:"),
    }
}

/// Gets the arbitrage value based on the `.env` file or sets a default value.
///
/// # Returns
///
/// A `f64` representing the arbitrage value, with a minimum value of 0.10.
///
/// # Example
///
/// ```
/// let arb_value = get_arb_value();
/// println!("Arb Value: {}", arb_value);
/// ```
pub(crate) fn get_arb_value() -> f64 {
    match get_dotenv_variable("ARB_VALUE") {
        Ok(val) => match val.parse::<f64>() {
            Ok(val) => {
                if val >= 0.10 {
                    val
                } else {
                    println!("Not a valid Arb Value, setting to 0.10");
                    0.10
                }
            }
            Err(_) => {
                println!("Not a valid Arb Value, setting to 0.10");
                0.10
            }
        },
        Err(_) => {
            println!("Arb Value not found, setting to 0.10");
            0.10
        }
    }
}

/// Gets the strike difference value based on the `.env` file or sets a default.
///
/// # Returns
///
/// A `f64` representing the strike difference value, with a default of 5.0 if invalid.
///
/// # Example
///
/// ```
/// let strike_dif = get_strike_dif_value();
/// println!("Strike Difference Value: {}", strike_dif);
/// ```
pub(crate) fn get_strike_dif_value() -> f64 {
    match get_dotenv_variable("STRIKE_DIF_VALUE") {
        Ok(val) => match val.parse::<f64>() {
            Ok(val) => {
                if val >= 0.0 {
                    val
                } else {
                    println!("Not a valid Strike Dif Value, setting to 5.0");
                    5.0
                }
            }
            Err(_) => {
                println!("Not a valid Strike Dif Value, setting to 5.0");
                5.0
            }
        },
        Err(_) => {
            println!("Strike Dif Value not found, setting to 5.0");
            5.0
        }
    }
}

/// Determines if the bot should run in test mode based on user input or the `.env` file.
///
/// # Returns
///
/// A `bool` where `true` means production mode, and `false` means test mode.
///
/// # Example
///
/// ```
/// let is_production = get_mode();
/// if is_production {
///     println!("Running in production mode.");
/// } else {
///     println!("Running in test mode.");
/// }
/// ```
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

/// Gets the number of seconds to sleep between retries, based on the `.env` file or user input.
///
/// # Returns
///
/// A `u64` representing the number of seconds to sleep (default is 60).
///
/// # Example
///
/// ```
/// let sleep_seconds = get_seconds_to_sleep();
/// println!("Sleeping for {} seconds.", sleep_seconds);
/// ```
pub(crate) fn get_seconds_to_sleep() -> u64 {
    match get_dotenv_variable("SECONDS_TO_SLEEP") {
        Ok(val) => {
            match val.parse::<u64>() {
                Ok(parsed_val) => {
                    if parsed_val >= 5 {
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
                    if parsed_input >= 5 {
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

/// Gets the discount value from the `.env` file or user input, limited to the range -0.15 to 0.15.
///
/// # Returns
///
/// A `f64` representing the discount value, with a default of 0.0 if invalid.
///
/// # Example
///
/// ```
/// let discount = get_discount_value();
/// println!("Discount value: {}", discount);
/// ```
pub(crate) fn get_discount_value() -> f64 {
    match get_dotenv_variable("DISCOUNT_VALUE") {
        Ok(val) => match val.parse::<f64>() {
            Ok(val) => {
                if val >= -0.15 && val <= 0.15 {
                    val
                } else {
                    println!("Not a valid Discount Value, setting to 0.0");
                    0.0
                }
            }
            Err(_) => {
                println!("Not a valid Discount Value, setting to 0.0");
                0.0
            }
        },
        Err(_) => {
            let input: String = get_user_input("Enter a Discount Value between -0.15 and 0.15:");
            match input.parse::<f64>() {
                Ok(val) => {
                    if val >= -0.15 && val <= 0.15 {
                        val
                    } else {
                        println!("Not a valid Discount Value, setting to 0.0");
                        0.0
                    }
                }
                Err(_) => {
                    println!("Not a valid Discount Value, setting to 0.0");
                    0.0
                }
            }
        }
    }
}

/// Gets the number of days for the bot to receive data, based on the `.env` file or user input.
///
/// # Returns
///
/// An `i64` representing the number of days (between 1 and 9), with a default of 5.
///
/// # Example
///
/// ```
/// let num_days = get_num_days();
/// println!("Receiving data for {} days.", num_days);
/// ```
pub(crate) fn get_num_days() -> i64 {
    match get_dotenv_variable("NUM_DAYS") {
        Ok(val) => match val.parse::<i64>() {
            Ok(parsed_val) => {
                if parsed_val > 0 && parsed_val < 10 {
                    parsed_val
                } else {
                    println!("Not a valid number in the range 1-9, setting to 5");
                    5
                }
            }
            Err(_) => {
                println!("Not a valid integer, setting to 5");
                5
            }
        },
        Err(_) => {
            let input: String = get_user_input("Enter number of days worth of data to receive:");
            match input.parse::<i64>() {
                Ok(parsed_input) => {
                    if parsed_input > 0 && parsed_input < 10 {
                        parsed_input
                    } else {
                        println!("Not a valid number in the range 1-9, setting to 5");
                        5
                    }
                }
                Err(_) => {
                    println!("Not a valid integer, setting to 5");
                    5
                }
            }
        }
    }
}

/// Gets the number of days offset for the bot, based on the `.env` file or user input.
///
/// # Returns
///
/// An `i64` representing the number of days offset (between 0 and 21), with a default of 0.
///
/// # Example
///
/// ```
/// let offset_days = get_num_days_offset();
/// println!("Offsetting {} days.", offset_days);
/// ```
pub(crate) fn get_num_days_offset() -> i64 {
    match get_dotenv_variable("NUM_DAYS_OFFSET") {
        Ok(val) => match val.parse::<i64>() {
            Ok(parsed_val) => {
                if parsed_val >= 0 && parsed_val <= 21 {
                    parsed_val
                } else {
                    println!("Not a valid number in the range 0-21, setting to 0");
                    0
                }
            }
            Err(_) => {
                println!("Not a valid integer, setting to 0");
                0
            }
        },
        Err(_) => {
            let input: String = get_user_input("Enter number of days offset:");
            match input.parse::<i64>() {
                Ok(parsed_input) => {
                    if parsed_input >= 0 && parsed_input <= 21 {
                        parsed_input
                    } else {
                        println!("Not a valid number in the range 0-21, setting to 0");
                        0
                    }
                }
                Err(_) => {
                    println!("Not a valid integer, setting to 0");
                    0
                }
            }
        }
    }
}

/// Checks if the US stock market is currently open based on the given `current_time`.
///
/// # Arguments
///
/// * `current_time` - The current time in UTC.
///
/// # Returns
///
/// A `bool` indicating if the market is open.
///
/// # Example
///
/// ```
/// let market_open = is_us_stock_market_open(Utc::now());
/// if market_open {
///     println!("The market is open.");
/// } else {
///     println!("The market is closed.");
/// }
/// ```
pub(crate) fn is_us_stock_market_open(current_time: chrono::DateTime<Utc>) -> bool {
    let ny_time: DateTime<chrono_tz::Tz> = current_time.with_timezone(&New_York);
    if ny_time.weekday() == Weekday::Sat || ny_time.weekday() == Weekday::Sun {
        return false;
    }
    let Some(market_open) = New_York
    .with_ymd_and_hms(ny_time.year(), ny_time.month(), ny_time.day(), 9, 30, 0)
    .single() else { return false };
    let Some(market_close) = New_York
    .with_ymd_and_hms(ny_time.year(), ny_time.month(), ny_time.day(), 15, 30, 0)
    .single() else { return false };
    ny_time >= market_open && ny_time <= market_close
}

/// Calculates the final number of orders and fills based on fill type and portfolio value.
///
/// # Arguments
///
/// * `fill` - A string representing the fill type.
/// * `port_val` - A `f64` representing the portfolio value.
///
/// # Returns
///
/// A tuple `(i32, i32)` representing the number of orders and fills.
///
/// # Example
///
/// ```
/// let (num_orders, num_fills) = calc_final_num_orders("1", 10000.0);
/// println!("Orders: {}, Fills: {}", num_orders, num_fills);
/// ```
pub(crate) fn calc_final_num_orders(fill: &str, port_val: f64) -> (i32, i32) {
    let num_times: i32 = (port_val / 800.0).floor() as i32;
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

/// Gets the optimal number of orders and fills based on the portfolio value.
///
/// # Arguments
///
/// * `portfolio_value` - A `f64` representing the portfolio value.
///
/// # Returns
///
/// A tuple `(i32, i32)` representing the ideal number of orders and fills.
///
/// # Example
///
/// ```
/// let (orders, fills) = get_optimal_num_orders(10000.0);
/// println!("Optimal Orders: {}, Fills: {}", orders, fills);
/// ```
fn get_optimal_num_orders(portfolio_value: f64) -> (i32, i32) {
    let num: i32 = (portfolio_value / 800.0).sqrt() as i32;
    if num > 9 {
        ((portfolio_value / 800.0 / 9.0).floor() as i32, 9)
    } else {
        (num, num)
    }
}

/// Calculates the number of days between two dates.
///
/// # Arguments
///
/// * `current_date` - The current date in `YYMMDD` format.
/// * `date` - The target date in `YYMMDD` format.
///
/// # Returns
///
/// An `i64` representing the difference in days between the two dates.
///
/// # Example
///
/// ```
/// let days_diff = calc_time_difference("240101", "240201");
/// println!("Days difference: {}", days_diff);
/// ```
pub(crate) fn calc_time_difference(current_date: &str, date: &str) -> i64 {
    let current_time: NaiveDate = NaiveDate::parse_from_str(current_date, "%y%m%d").unwrap();
    let future_time: NaiveDate = NaiveDate::parse_from_str(date, "%y%m%d").unwrap();
    ((current_time - future_time).num_hours() as f64 / 24.0 * -1.0) as i64
}

/// Calculates the rank value for a contract based on average ask, arbitrage value, and dates.
///
/// # Arguments
///
/// * `avg_ask` - The average ask price as `f64`.
/// * `arb_val` - The arbitrage value as `f64`.
/// * `current_date` - The current date in `YYMMDD` format.
/// * `date` - The target date in `YYMMDD` format.
///
/// # Returns
///
/// A `f64` representing the rank value of the contract.
///
/// # Example
///
/// ```
/// let rank = calc_rank_value(2.5, 0.10, "240101", "240201");
/// println!("Rank value: {}", rank);
/// ```
pub(crate) fn calc_rank_value(avg_ask: f64, arb_val: f64, current_date: &str, date: &str) -> f64 {
    let difference: i64 = calc_time_difference(current_date, date) + 1;
    (avg_ask * arb_val) / (difference as f64)
}

/// Predicts the maximum risk-free profit for a calendar spread.
///
/// # Arguments
///
/// * `strike` - A reference to the strike price as `&f64`.
/// * `arb_val` - The arbitrage value as `f64`.
///
/// # Returns
///
/// A `f64` representing the maximum risk-free profit.
///
/// # Example
///
/// ```
/// let profit = calendar_spread_risk_free_profit(&200.0, 0.10);
/// println!("Max risk-free profit: {}", profit);
/// ```
pub(crate) fn calendar_spread_risk_free_profit(strike: &f64, arb_val: f64) -> f64 {
    let max_loss: f64 = (strike / 200.0) * 0.03;
    arb_val - max_loss
}

/// Formats the strike price to a `String` with two decimal places, trimming any trailing zeros.
///
/// # Arguments
///
/// * `price` - The strike price as `f64`.
///
/// # Returns
///
/// A `String` representation of the strike price.
///
/// # Example
///
/// ```
/// let formatted_price = format_strike(100.50);
/// println!("Formatted strike: {}", formatted_price);
/// ```
pub(crate) fn format_strike(price: f64) -> String {
    let mut formatted = format!("{:.2}", price);
    while formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    formatted
}
