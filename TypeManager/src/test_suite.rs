use crate::utils::*;

#[test]
fn test_gcd() {
    // just test that gcd works ok
    assert_eq!(gcd(4, 1), 1);
    assert_eq!(gcd(4,4), 4);
    assert_eq!(gcd(7,9), 1);
}