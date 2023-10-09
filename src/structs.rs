use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub(crate) struct AuthResponse {
    pub(crate) status: String,
    pub(crate) sessionid: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChainResponse {
    pub(crate) rows: Vec<ChainRow>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChainRow {
    pub(crate) st: String,
    pub(crate) data: Vec<ChainData>,
    pub(crate) s: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChainData {
    pub(crate) v: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Opt {
    pub(crate) asz: f64,
    pub(crate) mkt: f64,
    pub(crate) bid: f64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Contract {
    pub(crate) strike: f64,
    pub(crate) mkt_price: f64,
    pub(crate) date: String,
    pub(crate) type_contract: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Contender {
    pub(crate) arb_val: f64,
    pub(crate) avg_ask: f64,
    pub(crate) type_spread: String,
    pub(crate) exp_date: String,
    pub(crate) rank_value: f64,
    pub(crate) contracts: Vec<Contract>,
}

impl Contender {
    pub(crate) fn action(&self, index: usize) -> &str {
        match self.type_spread.as_str() {
            "Calendar" => {
                if index == 0 {
                    "SELL"
                } else {
                    "UNKNOWN"
                }
            }
            "Butterfly" => {
                if index == 1 {
                    "SELL"
                } else {
                    "BUY "
                }
            }
            "Boxspread" => {
                if index % 2 == 1 {
                    "SELL"
                } else {
                    "BUY "
                }
            }
            _ => "UNKNOWN",
        }
    }

    pub(crate) fn multiplier(&self, num_fills: i32, index: usize) -> i32 {
        if self.type_spread == "Butterfly" && index == 1 {
            num_fills * 2
        } else {
            num_fills
        }
    }

    pub(crate) fn arb_value(&self) -> f64 {
        if self.type_spread == "Boxspread" {
            self.arb_val + 5.0
        } else {
            self.arb_val
        }
    }
}
