# Trading Bot in Rust

A trading bot implemented in Rust, designed to automate trading strategies using various options strategies such as Calendar, Butterfly, and Boxspread. This bot interacts with Interactive Brokers to execute trades based on defined conditions and strategies.

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

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any changes or improvements.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
