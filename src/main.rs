mod activetick;
mod helpers;
mod structs;

use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};

use activetick::ActiveTick;
use helpers::{
    calc_final_num_orders, get_api_key, get_fill_type, get_mode, get_option, get_password,
    get_username, is_us_stock_market_open, is_weekday,
};

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
                                        format!(
                                            "\tLeg {}: {} {} * {:.2}{} {} @ {:.2}",
                                            i + 1,
                                            contender.action(i),
                                            contender.multiplier(num_fills, i),
                                            contender.contracts[i].strike,
                                            &contender.contracts[i].type_contract,
                                            &contender.contracts[i].date,
                                            contender.contracts[i].mkt_price
                                        )
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
