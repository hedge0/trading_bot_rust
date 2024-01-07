use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Confirmation {
    pub(crate) confirmed: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AccountResponse {
    pub(crate) id: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SecDefResponse {
    #[serde(rename = "companyName")]
    pub(crate) company_name: String,
    pub(crate) conid: Option<String>,
    pub(crate) sections: Option<Vec<Section>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Section {
    #[serde(rename = "secType")]
    pub(crate) sec_type: String,
    pub(crate) months: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SecDefInfoResponse {
    pub(crate) conid: f64,
    #[serde(rename = "maturityDate")]
    pub(crate) maturity_date: String,
    pub(crate) right: String,
    pub(crate) strike: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MarketDataResponse {
    #[serde(rename = "conidEx")]
    pub(crate) conid_ex: String,
    #[serde(rename = "84")]
    pub(crate) field_84: Option<String>,
    #[serde(rename = "85")]
    pub(crate) field_85: Option<String>,
    #[serde(rename = "86")]
    pub(crate) field_86: Option<String>,
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
    #[serde(rename = "acctId")]
    pub(crate) acct_id: String,
    #[serde(rename = "conidex")]
    pub(crate) con_idex: String,
    #[serde(rename = "orderType")]
    pub(crate) order_type: String,
    #[serde(rename = "listingExchange")]
    pub(crate) listing_exchange: String,
    #[serde(rename = "outsideRTH")]
    pub(crate) outside_rth: bool,
    pub(crate) price: f64,
    pub(crate) side: String,
    pub(crate) ticker: String,
    pub(crate) tif: String,
    pub(crate) referrer: String,
    pub(crate) quantity: i32,
    #[serde(rename = "useAdaptive")]
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

#[derive(Serialize, Deserialize, Debug)]
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
}
