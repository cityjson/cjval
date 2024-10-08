use crate::cjval::CJValidator;
use cjval;

#[test]
fn invalid_values_overflow() {
    //-- with MultiSurface
    let sdata = std::fs::read_to_string("data/texture1.city.json").unwrap();
    let mut v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(!re["textures"].is_valid());

    //-- with Solid
    let sdata = std::fs::read_to_string("data/texture2.city.json").unwrap();
    v = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(!re["textures"].is_valid());
}

#[test]
fn valid_multisolid() {
    let sdata = std::fs::read_to_string("data/texture3.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["textures"].is_valid());
}

#[test]
fn valid_multisurface_inner_rings() {
    let sdata = std::fs::read_to_string("data/texture4.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["textures"].is_valid());
}

#[test]
fn valid_solid_with_null() {
    let sdata = std::fs::read_to_string("data/texture5.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["textures"].is_valid());
}

#[test]
fn invalid_with_float_as_indices() {
    let sdata = std::fs::read_to_string("data/texture6.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(!re["textures"].is_valid());
}

#[test]
fn valid_with_many_null() {
    let sdata = std::fs::read_to_string("data/texture7.city.json").unwrap();
    let v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["textures"].is_valid());
}
