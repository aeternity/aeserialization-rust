use std::{fmt, iter};

use serde::{Deserialize, de::{self, Visitor, MapAccess}, Deserializer};

use crate::data::datatype;

#[derive(Debug, Deserialize)]
struct Instructions {
    instruction: Vec<Instruction>
}

#[derive(Debug, Deserialize)]
struct Instruction {
    opname: String,
    opcode: u8,
    end_bb: bool,
    in_auth: bool,
    offchain: bool,
    gas: Gas,
    format: Vec<String>,
    constructor: String,
    arg_types: Vec<datatype::Type>,
    res_type: datatype::Type,
    documentation: String,
}

#[derive(Debug)]
enum Gas {
    Same(u64),
    Changed {
        iris: u64,
        lima: u64,
    }
}

impl<'de> Deserialize<'de> for Gas {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GasVisitor;

        impl<'de> Visitor<'de> for GasVisitor {
            type Value = Gas;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Gas")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v >= 0 {
                    Ok(Gas::Same(v as u64))
                } else {
                    Err(de::Error::invalid_value(de::Unexpected::Signed(v), &self))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let err_msg = "iris and lima must both be present";

                let entry1 = map.next_entry::<String, u64>()?
                    .ok_or_else(|| de::Error::custom(err_msg))?;
                let entry2 = map.next_entry::<String, u64>()?
                    .ok_or_else(|| de::Error::custom(err_msg))?;

                let (lima, iris) = {
                    if entry1.0 == "lima" && entry2.0 == "iris" {
                        (entry1.1, entry2.1)
                    } else if entry2.0 == "lima" && entry1.0 == "iris" {
                        (entry2.1, entry1.1)
                    } else {
                        Err(de::Error::custom(err_msg))?
                    }
                };

                Ok(Gas::Changed { iris, lima })
            }
        }

        deserializer.deserialize_any(GasVisitor)
    }
}

pub fn generate_fate_op_enum() -> std::io::Result<()> {
    let instructions: Instructions = {
        let contents = std::fs::read_to_string("fate.toml")
            .expect("File not found");
        toml::from_str(&contents)
            .expect("Failed to deserialize")
    };
    let mut file = String::from("use crate::data::value::Value;\n\n");
    file += "pub enum FateOp {\n";
    for i in instructions.instruction {
        if i.arg_types.is_empty() {
            file += format!("    {},\n", i.opname).as_str();
        } else {
            file += format!("    {}({}),\n", i.opname, iter::repeat("Value").take(i.arg_types.len()).collect::<Vec<&str>>().join(", ")).as_str();
        }
    }
    file += "}\n";
    std::fs::write("src/fate_op.rs", file)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let contents = std::fs::read_to_string("fate.toml").expect("File not found");
        let instructions: Instructions = toml::from_str(&contents).expect("Failed to deserialize");
        println!("contents: {:?}", instructions);
    }

    #[test]
    fn test_generate_file() -> std::io::Result<()> {
        generate_fate_op_enum()
    }
}
