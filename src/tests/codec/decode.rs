#[cfg(test)]

use crate::bencodex::codec::decode::Decodable;
use super::utils;

#[test]
fn spec_test() {
    let specs = utils::iter_spec().unwrap();
    for spec in specs {
        let mut buf: Vec<u8> = vec![];
        println!("---- SPEC [{}] ----", spec.name);
        println!("BVALUE: {:?}", spec.bvalue);
        let decoded = spec.encoded.decode();
        assert_eq!(decoded, spec.bvalue);
        println!("---- PASSED ----");
    }
}