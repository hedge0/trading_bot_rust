use std::{collections::HashMap, error::Error};

use chrono::{Datelike, Local};
use ordered_float::OrderedFloat;

use crate::{
    helpers::calc_rank_value,
    structs::{Contender, Contract, Opt, OrderBody, RequestDataStruct},
};

// Function that builds butterfly order body.
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
        price: -1.0 * (((contract.arb_val - discount_value.unwrap()) * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

// Function that builds boxspread order body.
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
            conids_map.as_ref().unwrap()[contract.contracts[3].date.as_str()] // Short right dated Put.
                [contract.contracts[3].type_contract.as_str()]
                [(&contract.contracts[3].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[2].date.as_str()] // Long right dated Call.
                [contract.contracts[2].type_contract.as_str()]
                [(&contract.contracts[2].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()] // Long current dated Put.
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()] // Short current dated Call.
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: -1.0 * (((contract.arb_val - discount_value.unwrap()) * 100.0).round() / 100.0),
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
) -> RequestDataStruct {
    let mut request_data: RequestDataStruct = RequestDataStruct { orders: Vec::new() };

    for contract in contender_contracts {
        match contract.type_spread.as_str() {
            "Butterfly" => {
                request_data.orders.push(build_butterfly_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            "Boxspread" => {
                request_data.orders.push(build_boxspread_order(
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

// Function that returns a slice of the top butterfly arbs.
pub(crate) fn get_butterfly_contenders(
    contracts_map: &HashMap<String, Opt>,
    dates_slice: &Vec<String>,
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
    conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
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
                            let current_contract_conid: &String = conids_map
                                .get(date)
                                .and_then(|ct| ct.get(contract_type))
                                .and_then(|ct| ct.get(current_strike.into()))
                                .ok_or("Error accessing current conid")?;
                            let current_contract: &Opt = contracts_map
                                .get(current_contract_conid)
                                .ok_or("Error accessing current contract")?;

                            let left_strike: &f64 = &contract_strikes[i - 1];
                            let left_contract_conid: &String = conids_map
                                .get(date)
                                .and_then(|ct| ct.get(contract_type))
                                .and_then(|ct| ct.get(left_strike.into()))
                                .ok_or("Error accessing left conid")?;
                            let left_contract: &Opt = contracts_map
                                .get(left_contract_conid)
                                .ok_or("Error accessing left contract")?;

                            let right_strike: &f64 = &contract_strikes[i + 1];
                            let right_contract_conid: &String = conids_map
                                .get(date)
                                .and_then(|ct| ct.get(contract_type))
                                .and_then(|ct| ct.get(right_strike.into()))
                                .ok_or("Error accessing right conid")?;
                            let right_contract: &Opt = contracts_map
                                .get(right_contract_conid)
                                .ok_or("Error accessing right contract")?;

                            let arb_val: f64 = (2.0 * current_contract.mkt)
                                - (left_contract.mkt + right_contract.mkt);

                            if arb_val > 0.4
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
    contracts_map: &HashMap<String, Opt>,
    dates_slice: &Vec<String>,
    strike_slice: &HashMap<String, HashMap<String, Vec<f64>>>,
    conids_map: &HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>,
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
                        let current_c_conid: &String = conids_map
                            .get(date)
                            .and_then(|c| c.get("C"))
                            .and_then(|c| c.get(current_strike_c.into()))
                            .ok_or("Error accessing current call conid")?;
                        let current_c: &Opt = contracts_map
                            .get(current_c_conid)
                            .ok_or("Error accessing current call contract")?;

                        let current_strike_p: &f64 = &ps[i];
                        let current_p_conid: &String = conids_map
                            .get(date)
                            .and_then(|p| p.get("P"))
                            .and_then(|p| p.get(current_strike_p.into()))
                            .ok_or("Error accessing current put conid")?;
                        let current_p: &Opt = contracts_map
                            .get(current_p_conid)
                            .ok_or("Error accessing current put contract")?;

                        let right_strike_c: &f64 = &cs[i + 1];
                        let right_c_conid: &String = conids_map
                            .get(date)
                            .and_then(|c| c.get("C"))
                            .and_then(|c| c.get(right_strike_c.into()))
                            .ok_or("Error accessing right call conid")?;
                        let right_c: &Opt = contracts_map
                            .get(right_c_conid)
                            .ok_or("Error accessing right call contract")?;

                        let right_strike_p: &f64 = &ps[i + 1];
                        let right_p_conid: &String = conids_map
                            .get(date)
                            .and_then(|p| p.get("P"))
                            .and_then(|p| p.get(right_strike_p.into()))
                            .ok_or("Error accessing right put conid")?;
                        let right_p: &Opt = contracts_map
                            .get(right_p_conid)
                            .ok_or("Error accessing right put contract")?;

                        let arb_val: f64 =
                            (current_p.mkt + right_c.mkt) - (current_c.mkt + right_p.mkt);

                        if arb_val < -5.4
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
                            let rank_value: f64 = calc_rank_value(
                                avg_ask,
                                (-1.0 * arb_val) - 5.0,
                                &current_date,
                                date,
                            );

                            contender_contracts.push(Contender {
                                arb_val: (-1.0 * arb_val * 100.0).round() / 100.0,
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
