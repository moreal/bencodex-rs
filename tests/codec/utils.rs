use num_bigint::BigInt;
use num_traits::cast::FromPrimitive;
use std::collections::BTreeMap;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;
use yaml_rust::parser::MarkedEventReceiver;
use yaml_rust::parser::Parser;
use yaml_rust::scanner::{Marker, ScanError, TokenType};
use yaml_rust::Event;

use bencodex::codec::types::{BencodexKey, BencodexValue};

#[derive(PartialEq, Debug)]
pub struct Spec {
    pub bvalue: BencodexValue,
    pub encoded: Vec<u8>,
    pub name: String,
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
                            "binary" => {
                                BencodexValue::Binary(base64::decode(v.replace('\n', "")).unwrap())
                            }
                            "null" => match v.as_ref() {
                                "~" | "null" => BencodexValue::Null(()),
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
                        BencodexValue::Null(())
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
                    if let None = cur_key {
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
                let encoded = match fs::read(path.to_owned()) {
                    Ok(v) => v,
                    Err(why) => panic!(why),
                };

                path.set_extension("yaml");
                let content = match fs::read_to_string(path.to_owned()) {
                    Ok(s) => s,
                    Err(why) => panic!(why),
                };

                let bvalue: BencodexValue =
                    match TestsuiteYamlLoader::load_from_str(&content.to_string()) {
                        Ok(v) => v.first().unwrap().to_owned(),
                        Err(why) => panic!(why),
                    };

                Spec {
                    bvalue: bvalue,
                    encoded: encoded,
                    name: path.file_name().unwrap().to_str().unwrap().to_owned(),
                }
            } else {
                unreachable!();
            }
        })
        .collect::<Vec<_>>();
    Ok(files)
}
