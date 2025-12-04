// engine_core/tests/test.rs
use engine_core::add;

#[test]
fn adds_two_numbers() {
    assert_eq!(add(2, 3), 5);
}