#[allow(dead_code)]
mod helpers;

#[cfg(test)]
mod tests {
    use chrono::{Datelike, LocalResult, TimeZone, Utc, Weekday};

    use crate::helpers::{
        calc_final_num_orders, calc_rank_value, calc_time_difference, convert_date,
        is_us_stock_market_open, is_weekday,
    };

    #[test]
    fn test_is_us_stock_market_open() {
        // Function to safely extract DateTime from LocalResult
        fn extract_datetime(result: LocalResult<chrono::DateTime<Utc>>) -> chrono::DateTime<Utc> {
            match result {
                LocalResult::Single(dt) => dt,
                _ => panic!("Invalid date/time provided."),
            }
        }

        // Test when market is definitely closed.
        let time = extract_datetime(Utc.with_ymd_and_hms(2023, 10, 10, 5, 0, 0)); // 5:00 AM UTC
        assert_eq!(is_us_stock_market_open(time), false);

        // Test right at market open.
        let time = extract_datetime(Utc.with_ymd_and_hms(2023, 10, 10, 9, 30, 0)); // 9:30 AM UTC
        assert_eq!(is_us_stock_market_open(time), true);

        // Test during open market hours.
        let time = extract_datetime(Utc.with_ymd_and_hms(2023, 10, 10, 12, 0, 0)); // 12:00 PM UTC
        assert_eq!(is_us_stock_market_open(time), true);

        // Test right at market close.
        let time = extract_datetime(Utc.with_ymd_and_hms(2023, 10, 10, 15, 15, 0)); // 3:15 PM UTC
        assert_eq!(is_us_stock_market_open(time), true);

        // Test right after market close.
        let time = extract_datetime(Utc.with_ymd_and_hms(2023, 10, 10, 15, 16, 0)); // 3:16 PM UTC
        assert_eq!(is_us_stock_market_open(time), false);
    }

    #[test]
    fn test_is_weekday() {
        // Obtain the current day of the week.
        let today: Weekday = Utc::now().weekday();

        // Determine if today is a weekday. If today is Saturday or Sunday, it should return false; otherwise, true.
        let expected: bool = today != Weekday::Sat && today != Weekday::Sun;

        // Check that the function's result matches the expected value.
        assert_eq!(is_weekday(), expected);
    }

    #[test]
    fn test_calc_final_num_orders() {
        // Test for port_val less than 600.
        assert_eq!(calc_final_num_orders("1", 599.0), (0, 0));

        // Test for fill type "1".
        assert_eq!(calc_final_num_orders("1", 600.0), (1, 1));
        assert_eq!(calc_final_num_orders("1", 1200.0), (1, 1));

        // Test for fill type "2".
        assert_eq!(calc_final_num_orders("2", 600.0), (1, 1));
        assert_eq!(calc_final_num_orders("2", 1200.0), (1, 2));

        // Test for fill type "3".
        assert_eq!(calc_final_num_orders("3", 600.0), (1, 1));
        assert_eq!(calc_final_num_orders("3", 1200.0), (2, 1));
    }

    #[test]
    fn test_calc_time_difference() {
        // Test with a difference of 1 day.
        // Current date: 220101, Date: 220102, Expected difference: 1 day.
        let difference = calc_time_difference("220101", "220102");
        assert_eq!(difference, 1);

        // Test with a difference of 5 days.
        // Current date: 220101, Date: 220106, Expected difference: 5 days.
        let difference = calc_time_difference("220101", "220106");
        assert_eq!(difference, 5);

        // Test with dates being the same.
        // Current date: 220101, Date: 220101, Expected difference: 0 days.
        let difference = calc_time_difference("220101", "220101");
        assert_eq!(difference, 0);

        // Test with the current date being later than the date.
        // Current date: 220106, Date: 220101, Expected difference: -5 days.
        let difference = calc_time_difference("220106", "220101");
        assert_eq!(difference, -5);
    }

    #[test]
    fn test_calc_rank_value() {
        // Test with a time difference of 1 day.
        // Current date: 220101, Date: 220102, avg_ask: 10.0, arb_val: 5.0, Expected rank value: 50.0.
        let rank_value = calc_rank_value(10.0, 5.0, "220101", "220102");

        assert!((rank_value - (50.0 / 2.0)).abs() < 1e-9); // Using a small epsilon for floating point comparison

        // Test with a time difference of 5 days.
        // Current date: 220101, Date: 220106, avg_ask: 10.0, arb_val: 5.0, Expected rank value: 12.5.
        let rank_value = calc_rank_value(10.0, 5.0, "220101", "220106");
        assert!((rank_value - (50.0 / 6.0)).abs() < 1e-9);

        // Test with dates being the same.
        // Current date: 220101, Date: 220101, avg_ask: 10.0, arb_val: 5.0, Expected rank value: 500.0.
        let rank_value = calc_rank_value(10.0, 5.0, "220101", "220101");
        assert!((rank_value - (50.0 / 1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_convert_date() {
        // Test date conversion for January of 2022.
        // Input date: "220101", Expected converted date: "JAN22".
        let converted_date = convert_date("220101");
        assert_eq!(converted_date, "JAN22");

        // Test date conversion for December of 2022.
        // Input date: "221231", Expected converted date: "DEC22".
        let converted_date = convert_date("221231");
        assert_eq!(converted_date, "DEC22");

        // Test date conversion for May of 2022.
        // Input date: "220515", Expected converted date: "MAY22".
        let converted_date = convert_date("220515");
        assert_eq!(converted_date, "MAY22");

        // Test date conversion for a leap year, February of 2024.
        // Input date: "240229", Expected converted date: "FEB24".
        let converted_date = convert_date("240229");
        assert_eq!(converted_date, "FEB24");
    }
}
