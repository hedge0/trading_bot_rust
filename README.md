# Options Arbitrage Trading Bot in Rust

## Overview

A high-performance trading bot implemented in Rust, designed to automate arbitrage strategies using various option spreads such as Calendar, Butterfly, and Boxspread. This bot interacts with the Interactive Brokers Web API to execute trades and pull live market data. For optimal performance, it's best to host the bot as close to IBKR's data centers as possible. I used an AWS EC2 instance with a New Jersey subnet and was able to identify all potential trades in approximately 100 ms.

## Prerequisites

- Docker
- Docker Compose

## Setup

1. Clone the repository:
    ```bash
    git clone https://github.com/hedge0/trading_bot_rust.git
    cd trading_bot_rust
    ```

2. Create a `.env` file in the root directory and add the required environment variables:
    ```env
    # Trading parameters
    TICKER=your_ticker_value
    OPTION=your_option_value
    FILL_TYPE=your_fill_type
    ARB_VALUE=your_arb_value
    STRIKE_DIF_VALUE=your_strike_dif_value
    DISCOUNT_VALUE=your_discount_value
    NUM_DAYS=your_num_days
    NUM_DAYS_OFFSET=your_num_days_offset

    # IBKR API credentials
    DOMAIN=your_ibkr_domain
    PORT=your_ibkr_port

    # Bot mode
    TEST_MODE=true_or_false

    # Miscellaneous
    SECONDS_TO_SLEEP=your_seconds_to_sleep
    ```

3. Build and run the Docker container:
    ```bash
    docker-compose up --build
    ```

## Usage

- The bot will automatically start and listen for market conditions to execute trades.
- Configure the bot by editing the `.env` file with your preferred settings and environment variables.
- The bot can be run in both live and testing modes.

## Trading Strategies

The bot supports the following trading strategies:
- **Calendar Spread**
- **Butterfly Spread**
- **Boxspread**

## Running Tests

To run the tests for the library, use the following command:

```bash
cargo test
```

The test file includes various test cases for functions such as:

- `test_get_dotenv_variable`
- `test_calc_final_num_orders`
- `test_calc_time_difference`
- `test_calc_rank_value`

These tests ensure the correctness of the functions used in the bot.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any changes or improvements.

## License

This project is licensed under the All Rights Reserved License - see the [LICENSE](LICENSE) file for details.
