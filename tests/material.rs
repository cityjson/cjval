use crate::cjval::CJValidator;
use cjval;

#[test]
fn invalid_values_overflow() {
    //-- with Solid
    let sdata = std::fs::read_to_string("data/material.city.json").unwrap();
    let mut v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(!re["materials"].is_valid());

    //-- with CompositeSurface
    let sdata = std::fs::read_to_string("data/material2.city.json").unwrap();
    v = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(!re["materials"].is_valid());
}

#[test]
fn valid() {
    //-- with Solid
    let sdata = std::fs::read_to_string("data/material3.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["materials"].is_valid());
}
