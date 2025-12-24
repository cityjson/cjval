use crate::cjval::CJValidator;
use cjval;

#[test]
fn valid() {
    let sdata = std::fs::read_to_string("data/japanese.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(re["extensions"].is_valid());
}
