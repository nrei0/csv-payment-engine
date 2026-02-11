use std::str::FromStr;

use rust_decimal::{Decimal, Error, RoundingStrategy};

const AMOUNT_SCALE: u32 = 4;

// Parses a string amount into fixed-point (scale 10^4).
// Values with more than 4 decimal places are truncated toward zero.
// For example, "123.4567" would be parsed into 1234567, and "0.0001" would be parsed into 1.
pub fn parse_amount_fixed_decimal(s: &str) -> Result<i128, Error> {
    let s = s.trim();
    let d = Decimal::from_str(s)?;

    // No rounding.
    let mut d = d.round_dp_with_strategy(AMOUNT_SCALE, RoundingStrategy::ToZero);
    d.rescale(AMOUNT_SCALE);

    Ok(d.mantissa())
}

// Formats an i128 amount in fixed-point format (scaled by 10^4) back into a string with up to 4 decimal places.
// For example, 1234567 would be formatted into "123.4567", and 1 would be formatted into "0.0001".
pub fn format_amount_to_string(v: i64) -> String {
    Decimal::new(v, AMOUNT_SCALE).normalize().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trims_whitespace() {
        assert_eq!(parse_amount_fixed_decimal(" 1.2345 ").unwrap(), 12_345);
        assert_eq!(parse_amount_fixed_decimal("\n\t0.0001\r").unwrap(), 1);
    }

    #[test]
    fn parse_integer_and_simple_decimals() {
        assert_eq!(parse_amount_fixed_decimal("0").unwrap(), 0);
        assert_eq!(parse_amount_fixed_decimal("1").unwrap(), 10_000);
        assert_eq!(parse_amount_fixed_decimal("1.0").unwrap(), 10_000);
        assert_eq!(parse_amount_fixed_decimal("1.2").unwrap(), 12_000);
        assert_eq!(parse_amount_fixed_decimal("1.23").unwrap(), 12_300);
        assert_eq!(parse_amount_fixed_decimal("1.234").unwrap(), 12_340);
        assert_eq!(parse_amount_fixed_decimal("1.2345").unwrap(), 12_345);
    }

    #[test]
    fn parse_leading_zeros_and_dot_forms() {
        assert_eq!(parse_amount_fixed_decimal("0001.2300").unwrap(), 12_300);
        assert_eq!(parse_amount_fixed_decimal("0.5").unwrap(), 5_000);
        assert_eq!(parse_amount_fixed_decimal(".5").unwrap(), 5_000);
        assert_eq!(parse_amount_fixed_decimal("1.").unwrap(), 10_000);
    }

    #[test]
    fn parse_negative_amounts() {
        assert_eq!(parse_amount_fixed_decimal("-1").unwrap(), -10_000);
        assert_eq!(parse_amount_fixed_decimal("-1.2345").unwrap(), -12_345);
        assert_eq!(parse_amount_fixed_decimal("-0.0001").unwrap(), -1);
    }

    #[test]
    fn parse_truncates_more_than_four_decimals_toward_zero() {
        assert_eq!(parse_amount_fixed_decimal("1.23456").unwrap(), 12_345);
        assert_eq!(parse_amount_fixed_decimal("1.23459").unwrap(), 12_345);

        assert_eq!(parse_amount_fixed_decimal("-1.23456").unwrap(), -12_345);
        assert_eq!(parse_amount_fixed_decimal("-0.00019").unwrap(), -1);
    }

    #[test]
    fn parse_rejects_invalid_inputs() {
        assert!(parse_amount_fixed_decimal("").is_err());
        assert!(parse_amount_fixed_decimal("   ").is_err());
        assert!(parse_amount_fixed_decimal("abc").is_err());
        assert!(parse_amount_fixed_decimal("1.2.3").is_err());
        assert!(parse_amount_fixed_decimal("--1").is_err());
        assert!(parse_amount_fixed_decimal("1e-3").is_err());
    }

    #[test]
    fn format_basic_values() {
        assert_eq!(format_amount_to_string(0), "0");
        assert_eq!(format_amount_to_string(10_000), "1");
        assert_eq!(format_amount_to_string(12_000), "1.2");
        assert_eq!(format_amount_to_string(12_300), "1.23");
        assert_eq!(format_amount_to_string(12_340), "1.234");
        assert_eq!(format_amount_to_string(12_345), "1.2345");
    }

    #[test]
    fn format_small_and_negative_values() {
        assert_eq!(format_amount_to_string(1), "0.0001");
        assert_eq!(format_amount_to_string(10), "0.001");
        assert_eq!(format_amount_to_string(100), "0.01");
        assert_eq!(format_amount_to_string(1000), "0.1");

        assert_eq!(format_amount_to_string(-1), "-0.0001");
        assert_eq!(format_amount_to_string(-10_000), "-1");
        assert_eq!(format_amount_to_string(-12_345), "-1.2345");
    }

    #[test]
    fn parse_format_round_trip_on_scaled_values() {
        let samples: [i64; 10] = [0, 1, 10, 100, 9999, 10_000, 12_345, 1_000_000, -1, -98_765];

        for &v in &samples {
            let s = format_amount_to_string(v);
            let parsed = parse_amount_fixed_decimal(&s).unwrap();
            assert_eq!(parsed, v as i128, "round-trip failed for v={v} s={s}");
        }
    }
}
