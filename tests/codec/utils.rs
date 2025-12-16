use base64::Engine;
use num_bigint::BigInt;
use num_traits::cast::FromPrimitive;
#[cfg(feature = "json")]
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use yaml_rust::Event;
use yaml_rust::parser::MarkedEventReceiver;
use yaml_rust::parser::Parser;
use yaml_rust::scanner::{Marker, ScanError, TokenType};

use bencodex::codec::types::{BencodexKey, BencodexValue};
#[cfg(feature = "json")]
use bencodex::json::BinaryEncoding;

#[derive(PartialEq, Debug)]
pub struct Spec {
    pub bvalue: BencodexValue,
    pub encoded: Vec<u8>,
    pub name: String,
}

#[derive(PartialEq, Debug)]
pub struct SpecWithJson {
    pub spec: Spec,
    pub json: String,
}

impl Deref for SpecWithJson {
    type Target = Spec;

    fn deref(&self) -> &Self::Target {
        &self.spec
    }
}

static SPEC_PATH: &str = "spec/testsuite";

struct TestsuiteYamlLoader {
    docs: Vec<BencodexValue>,
    // states
    // (current node, anchor_id) tuple
    key_stack: Vec<Option<BencodexKey>>,
    doc_stack: Vec<(BencodexValue, usize)>,
}

#[cfg(not(tarpaulin_include))]
impl MarkedEventReceiver for TestsuiteYamlLoader {
    fn on_event(&mut self, ev: Event, _: Marker) {
        match ev {
            Event::DocumentStart => {
                // do nothing
            }
            Event::DocumentEnd => match self.doc_stack.len() {
                1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                _ => unreachable!(),
            },
            Event::SequenceStart(aid) => {
                self.doc_stack.push((BencodexValue::List(Vec::new()), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::MappingStart(aid) => {
                self.key_stack.push(None);
                self.doc_stack
                    .push((BencodexValue::Dictionary(BTreeMap::new()), aid));
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::Scalar(v, _, aid, tag) => {
                let value = if let Some(TokenType::Tag(ref handle, ref suffix)) = tag {
                    // XXX tag:yaml.org,2002:
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => {
                                // "true" or "false"
                                match v.parse::<bool>() {
                                    Err(_) => unreachable!(),
                                    Ok(v) => BencodexValue::Boolean(v),
                                }
                            }
                            "int" => match v.parse::<i64>() {
                                Err(_) => unreachable!(),
                                Ok(v) => BencodexValue::Number(BigInt::from_i64(v).unwrap()),
                            },
                            "binary" => BencodexValue::Binary(
                                base64::engine::general_purpose::STANDARD
                                    .decode(v.replace('\n', ""))
                                    .unwrap(),
                            ),
                            "null" => match v.as_ref() {
                                "~" | "null" => BencodexValue::Null,
                                _ => unreachable!(),
                            },
                            _ => BencodexValue::Text(v),
                        }
                    } else {
                        BencodexValue::Text(v)
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    if let Ok(i) = BigInt::from_str(&v) {
                        BencodexValue::Number(i)
                    } else if let Ok(b) = v.parse::<bool>() {
                        BencodexValue::Boolean(b)
                    } else if v == "null" {
                        BencodexValue::Null
                    } else {
                        BencodexValue::Text(v)
                    }
                };

                self.insert_new_node((value, aid));
            }
            _ => { /* ignore */ }
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl TestsuiteYamlLoader {
    fn insert_new_node(&mut self, node: (BencodexValue, usize)) {
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (BencodexValue::List(ref mut v), _) => v.push(node.0),
                (BencodexValue::Dictionary(ref mut h), _) => {
                    let cur_key = self.key_stack.last().unwrap();
                    // current node is a key
                    if cur_key.is_none() {
                        self.key_stack.pop();
                        self.key_stack.push(match node.0 {
                            BencodexValue::Binary(v) => Some(BencodexKey::Binary(v)),
                            BencodexValue::Text(v) => Some(BencodexKey::Text(v)),
                            _ => unreachable!(),
                        });
                    // current node is a value
                    } else {
                        let newkey = self.key_stack.pop().unwrap().unwrap();
                        self.key_stack.push(None);
                        h.insert(newkey, node.0);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<BencodexValue>, ScanError> {
        let mut loader = TestsuiteYamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
        };
        let mut parser = Parser::new(source.chars());
        parser.load(&mut loader, true)?;
        Ok(loader.docs)
    }
}

#[cfg(feature = "json")]
#[derive(Debug)]
struct SpecImportError {
    #[allow(dead_code)]
    msg: String,
}

#[cfg(feature = "json")]
impl std::fmt::Display for SpecImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "json")]
impl SpecImportError {
    pub fn new<T: ToString>(msg: T) -> SpecImportError {
        SpecImportError {
            msg: msg.to_string(),
        }
    }
}

#[cfg(feature = "json")]
impl std::error::Error for SpecImportError {}

#[cfg(feature = "json")]
fn hex_to_base64(s: &String) -> String {
    if let Some(stripped) = s.strip_prefix("0x") {
        format!(
            "b64:{}",
            base64::engine::general_purpose::STANDARD.encode(hex::decode(stripped).unwrap())
        )
    } else {
        s.to_owned()
    }
}

#[cfg(feature = "json")]
fn base64_to_hex(s: &String) -> String {
    if let Some(stripped) = s.strip_prefix("b64:") {
        format!(
            "0x{}",
            hex::encode(
                base64::engine::general_purpose::STANDARD
                    .decode(stripped)
                    .unwrap()
            )
        )
    } else {
        s.to_owned()
    }
}

#[cfg(feature = "json")]
fn traverse(value: &Value, bytes_encode_method: &BinaryEncoding) -> Value {
    let converter = match bytes_encode_method {
        BinaryEncoding::Hex => base64_to_hex,
        BinaryEncoding::Base64 => hex_to_base64,
    };
    match value {
        Value::Object(obj) => {
            let mut ret = obj.clone();
            for (key, value) in obj.iter() {
                let new_key = converter(key);
                let new_value = match value {
                    Value::String(s) if s.starts_with("0x") => Value::String(hex_to_base64(s)),
                    _ => value.to_owned(),
                };

                if !new_key.eq(key) {
                    ret.remove(key);

                    ret.insert(new_key, new_value);
                } else {
                    ret[key] = new_value;
                }
            }

            Value::Object(ret)
        }
        Value::Array(list) => Value::Array(
            list.iter()
                .map(|x| match x {
                    Value::String(s) => Value::String(converter(s)),
                    _ => x.to_owned(),
                })
                .collect::<Vec<Value>>(),
        ),
        Value::String(s) => Value::String(converter(s)),
        _ => value.to_owned(),
    }
}

#[cfg(not(tarpaulin_include))]
pub fn iter_spec() -> std::io::Result<Vec<Spec>> {
    let files = fs::read_dir(SPEC_PATH)
        .unwrap()
        .filter(|entry| {
            if let Ok(file) = entry {
                println!("{:?}", file);
                if let Some(ext) = file.path().extension() {
                    ext == "dat"
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|entry: std::io::Result<DirEntry>| -> Spec {
            if let Ok(file) = entry {
                let mut path: PathBuf = file.path();
                let encoded = match fs::read(&path) {
                    Ok(v) => v,
                    Err(why) => panic!("{}", why),
                };

                path.set_extension("yaml");
                let content = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(why) => panic!("{}", why),
                };

                let bvalue: BencodexValue =
                    match TestsuiteYamlLoader::load_from_str(&content.to_string()) {
                        Ok(v) => v.first().unwrap().to_owned(),
                        Err(why) => panic!("{}", why),
                    };

                path.set_extension("repr.json");
                println!("path {:?}", path);

                Spec {
                    bvalue,
                    encoded,
                    name: path.file_name().unwrap().to_str().unwrap().to_owned(),
                }
            } else {
                unreachable!();
            }
        })
        .collect::<Vec<_>>();
    Ok(files)
}

#[cfg(feature = "json")]
#[cfg(not(tarpaulin_include))]
pub fn iter_spec_with_json(
    bytes_encode_method: BinaryEncoding,
) -> std::io::Result<Vec<SpecWithJson>> {
    use serde_json::Value;

    let files = fs::read_dir(SPEC_PATH)
        .unwrap()
        .filter(|entry| {
            if let Ok(file) = entry {
                println!("{:?}", file);
                if let Some(ext) = file.path().extension() {
                    ext == "dat"
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|entry: std::io::Result<DirEntry>| -> SpecWithJson {
            if let Ok(file) = entry {
                let mut path: PathBuf = file.path();
                let encoded = match fs::read(&path) {
                    Ok(v) => v,
                    Err(why) => panic!("{}", why),
                };

                path.set_extension("yaml");
                let content = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(why) => panic!("{}", why),
                };

                let bvalue: BencodexValue =
                    match TestsuiteYamlLoader::load_from_str(&content.to_string()) {
                        Ok(v) => v.first().unwrap().to_owned(),
                        Err(why) => panic!("{}", why),
                    };

                path.set_extension("repr.json");
                println!("path {:?}", path);

                let json = match fs::read_to_string(&path)
                    .map_err(|_| SpecImportError::new("Failed to read json."))
                    .and_then(|x| {
                        x.parse::<Value>()
                            .map_err(|_| SpecImportError::new("Failed to parse json."))
                    })
                    .map(|x| traverse(&x, &bytes_encode_method))
                    .and_then(|x| {
                        serde_json::to_string(&x)
                            .map_err(|_| SpecImportError::new("Failed to stringify json."))
                    }) {
                    Ok(s) => s,
                    Err(why) => panic!("{}", why),
                };

                SpecWithJson {
                    spec: Spec {
                        bvalue,
                        encoded,
                        name: path.file_name().unwrap().to_str().unwrap().to_owned(),
                    },
                    json,
                }
            } else {
                unreachable!();
            }
        })
        .collect::<Vec<_>>();
    Ok(files)
}
