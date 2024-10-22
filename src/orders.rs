use crate::structs::{Contender, OrderBody, RequestDataStruct};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

/// Builds the order body for a calendar spread.
///
/// # Arguments
///
/// * `contract` - A reference to a `Contender` that represents the contract.
/// * `num_fills` - The number of fills required for the order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the order.
///
/// # Returns
///
/// An `OrderBody` representing the order for the calendar spread.
pub(crate) fn build_calendar_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    let order_val: f64 = contract.arb_val;
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
        price: -1.0 * (((order_val - discount_value.unwrap()) * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

/// Builds the order body for a butterfly bull spread.
///
/// # Arguments
///
/// * `contract` - A reference to a `Contender` that represents the contract.
/// * `num_fills` - The number of fills required for the order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the order.
///
/// # Returns
///
/// An `OrderBody` representing the order for the butterfly bull spread.
pub(crate) fn build_butterfly_bull_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    let order_val: f64 = contract.contracts[0].mkt_price - contract.contracts[1].mkt_price;
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-1,{}/1",
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()]
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: ((order_val + discount_value.unwrap()) * 100.0).round() / 100.0,
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

/// Builds the order body for a butterfly bear spread.
///
/// # Arguments
///
/// * `contract` - A reference to a `Contender` that represents the contract.
/// * `num_fills` - The number of fills required for the order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the order.
///
/// # Returns
///
/// An `OrderBody` representing the order for the butterfly bear spread.
pub(crate) fn build_butterfly_bear_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    let order_val: f64 = contract.contracts[2].mkt_price - contract.contracts[1].mkt_price;
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-1,{}/1",
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[2].date.as_str()]
                [contract.contracts[2].type_contract.as_str()]
                [(&contract.contracts[2].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: (((order_val + discount_value.unwrap()) * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

/// Builds the order body for a boxspread put spread.
///
/// # Arguments
///
/// * `contract` - A reference to a `Contender` that represents the contract.
/// * `num_fills` - The number of fills required for the order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the order.
///
/// # Returns
///
/// An `OrderBody` representing the order for the boxspread put spread.
pub(crate) fn build_boxspread_put_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    let order_val: f64 = contract.contracts[0].mkt_price - contract.contracts[3].mkt_price;
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/-1,{}/1",
            conids_map.as_ref().unwrap()[contract.contracts[3].date.as_str()]
                [contract.contracts[3].type_contract.as_str()]
                [(&contract.contracts[3].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[0].date.as_str()]
                [contract.contracts[0].type_contract.as_str()]
                [(&contract.contracts[0].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: (((order_val + discount_value.unwrap()) * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

/// Builds the order body for a boxspread call spread.
///
/// # Arguments
///
/// * `contract` - A reference to a `Contender` that represents the contract.
/// * `num_fills` - The number of fills required for the order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the order.
///
/// # Returns
///
/// An `OrderBody` representing the order for the boxspread call spread.
pub(crate) fn build_boxspread_call_order(
    contract: &Contender,
    num_fills: i32,
    account_id: &Option<String>,
    conids_map: &Option<HashMap<String, HashMap<String, HashMap<OrderedFloat<f64>, String>>>>,
    discount_value: Option<f64>,
) -> OrderBody {
    let order_val: f64 = contract.contracts[2].mkt_price - contract.contracts[1].mkt_price;
    OrderBody {
        acct_id: account_id.clone().unwrap(),
        con_idex: format!(
            "28812380;;;{}/1,{}/-1",
            conids_map.as_ref().unwrap()[contract.contracts[2].date.as_str()]
                [contract.contracts[2].type_contract.as_str()]
                [(&contract.contracts[2].strike).into()],
            conids_map.as_ref().unwrap()[contract.contracts[1].date.as_str()]
                [contract.contracts[1].type_contract.as_str()]
                [(&contract.contracts[1].strike).into()]
        ),
        order_type: "LMT".to_string(),
        listing_exchange: "SMART".to_string(),
        outside_rth: false,
        price: (((order_val + discount_value.unwrap()) * 100.0).round() / 100.0),
        side: "BUY".to_string(),
        ticker: "SPX".to_string(),
        tif: "DAY".to_string(),
        referrer: "NO_REFERRER_PROVIDED".to_string(),
        quantity: num_fills,
        use_adaptive: false,
    }
}

/// Builds the request data structure that includes the orders for each contender contract.
///
/// # Arguments
///
/// * `contender_contracts` - A vector of `Contender` contracts.
/// * `num_fills` - The number of fills for each order.
/// * `account_id` - An `Option<String>` containing the account ID.
/// * `conids_map` - A reference to an optional mapping of conids.
/// * `discount_value` - An optional discount value applied to the orders.
///
/// # Returns
///
/// A `RequestDataStruct` containing the order bodies for each contender contract.
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
            "Calendar" => {
                request_data.orders.push(build_calendar_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            "Butterfly" => {
                request_data.orders.push(build_butterfly_bull_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
                request_data.orders.push(build_butterfly_bear_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
            }
            "Boxspread" => {
                request_data.orders.push(build_boxspread_put_order(
                    contract,
                    num_fills,
                    account_id,
                    conids_map,
                    discount_value,
                ));
                request_data.orders.push(build_boxspread_call_order(
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
