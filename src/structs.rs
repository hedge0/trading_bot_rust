use simd_json::prelude::*;
use simd_json::serde::prelude::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct AccountResponse {
    pub(crate) id: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SecDefResponse {
    #[serde(rename = "companyName")]
    pub(crate) company_name: String,
    pub(crate) conid: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SecDefInfoResponse {
    pub(crate) conid: f64,
    #[serde(rename = "tradingClass")]
    pub(crate) trading_class: String,
    #[serde(rename = "maturityDate")]
    pub(crate) maturity_date: String,
    pub(crate) right: String,
    pub(crate) strike: f64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PortfolioResponse {
    #[serde(rename = "equitywithloanvalue")]
    pub(crate) equity_with_loan_value: PortfolioAmount,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PortfolioAmount {
    pub(crate) amount: f64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OrderBody {
    pub(crate) acct_id: String,
    pub(crate) con_idex: String,
    pub(crate) order_type: String,
    pub(crate) listing_exchange: String,
    pub(crate) outside_rth: bool,
    pub(crate) price: f64,
    pub(crate) side: String,
    pub(crate) ticker: String,
    pub(crate) tif: String,
    pub(crate) referrer: String,
    pub(crate) quantity: i32,
    pub(crate) use_adaptive: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RequestDataStruct {
    pub(crate) orders: Vec<OrderBody>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Order {
    pub(crate) status: String,
    pub(crate) order_id: f64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OrdersResponse {
    pub(crate) orders: Vec<Order>,
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
                    "BUY"
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
