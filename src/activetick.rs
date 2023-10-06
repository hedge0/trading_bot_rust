impl ActiveTick {
    fn init(
        &mut self,
        username: &str,
        password: &str,
        apikey: &str,
        num_days: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.username = username.to_string();
        self.password = password.to_string();
        self.apikey = apikey.to_string();
        self.num_days = std::time::Duration::from_secs(num_days * 24 * 60 * 60);
        let (dates, strikes) = self.get_spx_dates_and_strikes(self.get_session_id())?;
        self.dates_slice = dates;
        self.strike_slice = strikes;
        Ok(())
    }

    fn get_dates_slice(&self) -> &Vec<String> {
        &self.dates_slice
    }

    fn get_strike_slice(&self) -> &HashMap<String, HashMap<String, Vec<f64>>> {
        &self.strike_slice
    }

    fn get_session_id(&self) -> String {
        let auth_url = "https://api.activetick.com/authorize.json";
        let auth_params: HashMap<&str, &str> = [
            ("username", &self.username),
            ("password", &self.password),
            ("apikey", &self.apikey),
        ]
        .iter()
        .cloned()
        .collect();

        let client = Client::new();
        let response = client
            .get(auth_url)
            .query(&auth_params)
            .send()
            .expect("Error sending request");

        if response.status() == reqwest::StatusCode::OK {
            let auth_results: AuthResponse = response.json().expect("Error parsing JSON");
            if auth_results.status == "ok" {
                if let Some(session_id) = auth_results.sessionid {
                    return session_id;
                }
            } else {
                eprintln!("Error: User Unauthorized");
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: {}", response.status());
            std::process::exit(1);
        }

        "".to_string()
    }

    fn get_spx_dates_and_strikes(
        &self,
        session_id: String,
    ) -> Result<(Vec<String>, HashMap<String, HashMap<String, Vec<f64>>>), Box<dyn Error>> {
        let chain_url = "https://api.activetick.com/chain.json";
        let current_time = std::time::SystemTime::now();
        let future_time = current_time + self.num_days;
        let formatted_time = current_time
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .to_string();
        let formatted_future_time = future_time
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .to_string();

        let mut chain_params: HashMap<&str, String> = HashMap::new();
        chain_params.insert("sessionid", session_id);
        chain_params.insert("key", "SPXW_S U".to_string());
        chain_params.insert("chaintype", "equity_options".to_string());
        chain_params.insert("columns", "b,a,asz".to_string());
        chain_params.insert("begin_maturity_time", formatted_time);
        chain_params.insert("end_maturity_time", formatted_future_time);
        chain_params.insert("ignore_empty", "false".to_string());

        let client = Client::new();
        let response = client
            .get(chain_url)
            .query(&chain_params)
            .send()
            .expect("Error sending request");

        if response.status() == reqwest::StatusCode::OK {
            let chain_results: ChainResponse = response.json().expect("Error parsing JSON");
            if chain_results.rows.is_empty() {
                eprintln!("Error: SPX rows data is empty");
                std::process::exit(1);
            }

            let mut dates_slice = Vec::new();
            let mut strike_slice = HashMap::new();

            for v in chain_results.rows.iter() {
                if v.st == "ok" {
                    let code_parts: Vec<&str> = v.s.split('_').collect();
                    let exp_date = code_parts[1][..6].to_string();
                    let type_opt = code_parts[1][6..7].to_string();
                    let strike_str = code_parts[1][7..code_parts[1].len() - 3].to_string();
                    let strike = strike_str.parse::<f64>()?;
                    strike_slice.entry(exp_date.clone()).or_insert_with(|| {
                        dates_slice.push(exp_date.clone());
                        HashMap::new()
                    });
                    strike_slice
                        .get_mut(&exp_date)
                        .unwrap()
                        .entry(type_opt.clone())
                        .or_insert(Vec::new())
                        .push(strike);
                }
            }

            for key in strike_slice.keys() {
                if let Some(strikes) = strike_slice.get_mut(key) {
                    if let Some(call_strikes) = strikes.get_mut("C") {
                        call_strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    }
                    if let Some(put_strikes) = strikes.get_mut("P") {
                        put_strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    }
                }
            }

            return Ok((dates_slice, strike_slice));
        } else {
            eprintln!("Error: {}", response.status());
            std::process::exit(1);
        }
    }

    fn get_spx_data(
        &self,
        session_id: &str,
    ) -> HashMap<String, HashMap<String, HashMap<f64, Opt>>> {
        let chain_url = "https://api.activetick.com/chain.json";
        let current_time = SystemTime::now();
        let future_time = current_time + self.num_days;
        let formatted_time = format!(
            "{}",
            current_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let formatted_future_time = format!(
            "{}",
            future_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let mut chain_params = HashMap::new();
        chain_params.insert("sessionid", session_id.to_string());
        chain_params.insert("key", "SPXW_S U".to_string());
        chain_params.insert("chaintype", "equity_options".to_string());
        chain_params.insert("columns", "b,a,asz".to_string());
        chain_params.insert("begin_maturity_time", formatted_time);
        chain_params.insert("end_maturity_time", formatted_future_time);
        chain_params.insert("ignore_empty", "false".to_string());

        let mut contracts_map = HashMap::new();

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(chain_url)
            .query(&chain_params)
            .send()
            .expect("Error sending request");

        if response.status() == reqwest::StatusCode::OK {
            let chain_results: ChainResponse = response.json().expect("Error parsing JSON");
            if chain_results.rows.is_empty() {
                eprintln!("Error: SPX rows data is empty");
                std::process::exit(1);
            }

            for v in chain_results.rows.iter() {
                if v.st == "ok" {
                    let code_parts: Vec<&str> = v.s.split('_').collect();
                    let exp_date = code_parts[1][..6].to_string();
                    let type_opt = code_parts[1][6..7].to_string();
                    let strike_str = code_parts[1][7..code_parts[1].len() - 3].to_string();
                    let strike = strike_str.parse::<f64>().unwrap();
                    let bid = v.data[0].v.parse::<f64>().unwrap();
                    let ask = v.data[1].v.parse::<f64>().unwrap();
                    let asz_val = v.data[2].v.parse::<f64>().unwrap();
                    let mkt_val = ((bid + ask) / 2.0).round() / 100.0;

                    contracts_map
                        .entry(exp_date.clone())
                        .or_insert_with(|| HashMap::new());

                    contracts_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .entry(type_opt.clone())
                        .or_insert_with(|| HashMap::new());

                    contracts_map
                        .get_mut(&exp_date)
                        .unwrap()
                        .get_mut(&type_opt)
                        .unwrap()
                        .insert(
                            strike,
                            Opt {
                                asz: asz_val,
                                mkt: mkt_val,
                                bid,
                            },
                        );
                }
            }
        } else {
            eprintln!("Error: {}", response.status());
            std::process::exit(1);
        }

        contracts_map
    }

    // Function that returns a slice of the top calendar arbs
    fn get_calendar_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<f64, Opt>>>,
    ) -> Vec<Contender> {
        let mut contender_contracts = Vec::new();
        let current_date = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        for date_index in 0..(self.dates_slice.len() - 1) {
            let date = &self.dates_slice[date_index];
            for strike in &self.strike_slice[date]["C"] {
                let current_c = contracts_map[date]["C"][strike];
                let next_date = &self.dates_slice[date_index + 1];
                let (next_c, strike_exists) = match contracts_map[next_date]["C"].get(strike) {
                    Some(next_c) => (next_c, true),
                    None => (&Opt::default(), false),
                };

                if strike_exists {
                    let arb_val = current_c.mkt - next_c.mkt;

                    if arb_val > 0.15
                        && current_c.bid > 0.25
                        && next_c.bid > 0.25
                        && current_c.asz > 0.0
                        && next_c.asz > 0.0
                        && calc_time_difference(date, next_date) == 2
                    {
                        let avg_ask = ((current_c.asz + next_c.asz) / 2.0).round();
                        let rank_value = calc_rank_value(avg_ask, arb_val, &current_date, date);

                        contender_contracts.push(Contender {
                            arb_val,
                            avg_ask,
                            type_spread: "Calendar".to_string(),
                            exp_date: date.clone(),
                            rank_value,
                            contracts: vec![
                                Contract {
                                    strike: *strike,
                                    mkt_price: current_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                                Contract {
                                    strike: *strike,
                                    mkt_price: next_c.mkt,
                                    date: next_date.clone(),
                                    type_contract: "C".to_string(),
                                },
                            ],
                        });
                    }
                }
            }

            for strike in &self.strike_slice[date]["P"] {
                let current_p = contracts_map[date]["P"][strike];
                let next_date = &self.dates_slice[date_index + 1];
                let (next_p, strike_exists) = match contracts_map[next_date]["P"].get(strike) {
                    Some(next_p) => (next_p, true),
                    None => (&Opt::default(), false),
                };

                if strike_exists {
                    let arb_val = current_p.mkt - next_p.mkt;

                    if arb_val > 0.15
                        && current_p.bid > 0.25
                        && next_p.bid > 0.25
                        && current_p.asz > 0.0
                        && next_p.asz > 0.0
                        && calc_time_difference(date, next_date) == 2
                    {
                        let avg_ask = ((current_p.asz + next_p.asz) / 2.0).round();
                        let rank_value = calc_rank_value(avg_ask, arb_val, &current_date, date);

                        contender_contracts.push(Contender {
                            arb_val,
                            avg_ask,
                            type_spread: "Calendar".to_string(),
                            exp_date: date.clone(),
                            rank_value,
                            contracts: vec![
                                Contract {
                                    strike: *strike,
                                    mkt_price: current_p.mkt,
                                    date: date.clone(),
                                    type_contract: "P".to_string(),
                                },
                                Contract {
                                    strike: *strike,
                                    mkt_price: next_p.mkt,
                                    date: next_date.clone(),
                                    type_contract: "P".to_string(),
                                },
                            ],
                        });
                    }
                }
            }
        }

        contender_contracts
    }

    // Function that returns a slice of the top butterfly arbs
    fn get_butterfly_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<f64, Opt>>>,
    ) -> Vec<Contender> {
        let mut contender_contracts = Vec::new();
        let current_date = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        for date in &self.dates_slice {
            if self.strike_slice[date]["C"].len() > 2 {
                for i in 1..(self.strike_slice[date]["C"].len() - 1) {
                    let current_strike_c = self.strike_slice[date]["C"][i];
                    let current_c = &contracts_map[date]["C"][&current_strike_c];
                    let left_strike_c = self.strike_slice[date]["C"][i - 1];
                    let left_c = &contracts_map[date]["C"][&left_strike_c];
                    let right_strike_c = self.strike_slice[date]["C"][i + 1];
                    let right_c = &contracts_map[date]["C"][&right_strike_c];

                    let arb_val = (2.0 * current_c.mkt) - (left_c.mkt + right_c.mkt);

                    if arb_val > 0.15
                        && left_c.bid > 0.25
                        && right_c.bid > 0.25
                        && current_c.bid > 0.25
                        && left_c.asz > 0.0
                        && right_c.asz > 0.0
                        && current_c.asz > 0.0
                        && (current_strike_c - left_strike_c).round() == 5.0
                        && (right_strike_c - current_strike_c).round() == 5.0
                    {
                        let avg_ask =
                            ((left_c.asz + right_c.asz + (2.0 * current_c.asz)) / 4.0).round();
                        let rank_value = calc_rank_value(avg_ask, arb_val, &current_date, date);

                        contender_contracts.push(Contender {
                            arb_val,
                            avg_ask,
                            type_spread: "Butterfly".to_string(),
                            exp_date: date.clone(),
                            rank_value,
                            contracts: vec![
                                Contract {
                                    strike: left_strike_c,
                                    mkt_price: left_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                                Contract {
                                    strike: current_strike_c,
                                    mkt_price: current_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                                Contract {
                                    strike: right_strike_c,
                                    mkt_price: right_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                            ],
                        });
                    }
                }
            }

            if self.strike_slice[date]["P"].len() > 2 {
                for i in 1..(self.strike_slice[date]["P"].len() - 1) {
                    let current_strike_p = self.strike_slice[date]["P"][i];
                    let current_p = &contracts_map[date]["P"][&current_strike_p];
                    let left_strike_p = self.strike_slice[date]["P"][i - 1];
                    let left_p = &contracts_map[date]["P"][&left_strike_p];
                    let right_strike_p = self.strike_slice[date]["P"][i + 1];
                    let right_p = &contracts_map[date]["P"][&right_strike_p];

                    let arb_val = (2.0 * current_p.mkt) - (left_p.mkt + right_p.mkt);

                    if arb_val > 0.15
                        && left_p.bid > 0.25
                        && right_p.bid > 0.25
                        && current_p.bid > 0.25
                        && left_p.asz > 0.0
                        && right_p.asz > 0.0
                        && current_p.asz > 0.0
                        && (current_strike_p - left_strike_p).round() == 5.0
                        && (right_strike_p - current_strike_p).round() == 5.0
                    {
                        let avg_ask =
                            ((left_p.asz + right_p.asz + (2.0 * current_p.asz)) / 4.0).round();
                        let rank_value = calc_rank_value(avg_ask, arb_val, &current_date, date);

                        contender_contracts.push(Contender {
                            arb_val,
                            avg_ask,
                            type_spread: "Butterfly".to_string(),
                            exp_date: date.clone(),
                            rank_value,
                            contracts: vec![
                                Contract {
                                    strike: left_strike_p,
                                    mkt_price: left_p.mkt,
                                    date: date.clone(),
                                    type_contract: "P".to_string(),
                                },
                                Contract {
                                    strike: current_strike_p,
                                    mkt_price: current_p.mkt,
                                    date: date.clone(),
                                    type_contract: "P".to_string(),
                                },
                                Contract {
                                    strike: right_strike_p,
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

        contender_contracts
    }

    // Function that returns a slice of the top boxspread arbs
    fn get_boxspread_contenders(
        &self,
        contracts_map: &HashMap<String, HashMap<String, HashMap<f64, Opt>>>,
    ) -> Vec<Contender> {
        let mut contender_contracts = Vec::new();
        let current_date = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        for date in &self.dates_slice {
            if self.strike_slice[date]["C"].len() > 1 && self.strike_slice[date]["P"].len() > 1 {
                for i in 0..(self.strike_slice[date]["C"].len() - 1) {
                    let current_strike_c = self.strike_slice[date]["C"][i];
                    let current_c = &contracts_map[date]["C"][&current_strike_c];
                    let current_strike_p = self.strike_slice[date]["P"][i];
                    let current_p = &contracts_map[date]["P"][&current_strike_p];
                    let right_strike_c = self.strike_slice[date]["C"][i + 1];
                    let right_c = &contracts_map[date]["C"][&right_strike_c];
                    let right_strike_p = self.strike_slice[date]["P"][i + 1];
                    let right_p = &contracts_map[date]["P"][&right_strike_p];

                    let arb_val =
                        (current_c.mkt + right_p.mkt) - (current_p.mkt + right_c.mkt) - 5.0;

                    if arb_val > 0.15
                        && current_c.bid > 0.25
                        && current_p.bid > 0.25
                        && right_c.bid > 0.25
                        && right_p.bid > 0.25
                        && current_c.asz > 0.0
                        && current_p.asz > 0.0
                        && right_c.asz > 0.0
                        && right_p.asz > 0.0
                        && (right_strike_c - current_strike_c).round() == 5.0
                        && (right_strike_p - current_strike_p).round() == 5.0
                    {
                        let avg_ask = ((current_c.asz + right_c.asz + current_p.asz + right_p.asz)
                            / 4.0)
                            .round();
                        let rank_value = calc_rank_value(avg_ask, arb_val, &current_date, date);

                        contender_contracts.push(Contender {
                            arb_val,
                            avg_ask,
                            type_spread: "Boxspread".to_string(),
                            exp_date: date.clone(),
                            rank_value,
                            contracts: vec![
                                Contract {
                                    strike: current_strike_p,
                                    mkt_price: current_p.mkt,
                                    date: date.clone(),
                                    type_contract: "P".to_string(),
                                },
                                Contract {
                                    strike: current_strike_c,
                                    mkt_price: current_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                                Contract {
                                    strike: right_strike_c,
                                    mkt_price: right_c.mkt,
                                    date: date.clone(),
                                    type_contract: "C".to_string(),
                                },
                                Contract {
                                    strike: right_strike_p,
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

        contender_contracts
    }

    // Function that returns a slice of the top arbs given the number of orders
    fn get_contender_contracts(&self, option: &str, num_orders: usize) -> Vec<Contender> {
        let contracts_map = self.get_spx_data(&self.get_session_id());
        let mut contender_contracts_total = Vec::new();

        match option {
            "1" => contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map)),
            "2" => contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map)),
            "3" => contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map)),
            "4" => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map));
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map));
            }
            "5" => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map));
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map));
            }
            "6" => {
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map));
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map));
            }
            _ => {
                contender_contracts_total.extend(self.get_calendar_contenders(&contracts_map));
                contender_contracts_total.extend(self.get_butterfly_contenders(&contracts_map));
                contender_contracts_total.extend(self.get_boxspread_contenders(&contracts_map));
            }
        }

        contender_contracts_total.sort_by(|a, b| b.rank_value.partial_cmp(&a.rank_value).unwrap());

        if contender_contracts_total.len() > num_orders {
            contender_contracts_total.truncate(num_orders);
        }

        contender_contracts_total
    }
}
