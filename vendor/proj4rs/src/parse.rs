//! Numeric parsing uses the same strict Rust implementation on every target.
//! Local portability patch: upstream selected JavaScript prefix parsers on WASM.
//! Rejecting trailing text and integer overflow consistently avoids host-dependent CRS parameters.
pub use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::FromStr;
    #[test]
    fn strict_parameters_reject_trailing_text_and_integer_overflow() {
        assert_eq!(f64::from_str("1.25e2").unwrap(), 125.);
        assert!(f64::from_str("1.25junk").is_err());
        assert!(i32::from_str("2147483648").is_err());
        assert!(i32::from_str("2.5").is_err());
        assert!(bool::from_str("TRUE").is_err());
    }
}
