use super::utils;
#[cfg(test)]
use bencodex::codec::encode::Encode;
use bencodex::json::encode::BinaryEncoding;

#[test]
fn spec_test() {
    let specs = utils::iter_spec(BinaryEncoding::Base64).unwrap();
    for spec in specs {
        let mut buf: Vec<u8> = vec![];
        println!("---- SPEC [{}] ----", spec.name);
        println!("BVALUE: {:?}", spec.bvalue);
        spec.bvalue.clone().encode(&mut buf).ok();
        assert_eq!(buf, spec.encoded);

        println!("---- PASSED ----");
    }
}
