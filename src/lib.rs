#[allow(dead_code)]
mod helpers;

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Utc, Weekday};

    use crate::helpers::{
        calc_final_num_orders, calc_rank_value, calc_time_difference, is_weekday,
    };

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
}
