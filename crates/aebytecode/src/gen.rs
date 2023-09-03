use std::{fmt, iter};

use serde::{Deserialize, de::{self, Visitor, MapAccess}, Deserializer};

use crate::data::types;

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
    arg_types: Vec<types::Type>,
    res_type: types::Type,
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

pub fn generate_instructions_enum() -> std::io::Result<()> {
    let instructions: Instructions = {
        let contents = std::fs::read_to_string("fate.toml")
            .expect("File not found");
        toml::from_str(&contents)
            .expect("Failed to deserialize")
    };
    let mut file = String::from("use crate::code2::Arg;\n\n");
    file += "pub enum AddressingMode {\n";
    file += "    Short(u8),\n";
    file += "    Long {\n";
    file += "        high: u8,\n";
    file += "        low: u8,\n";
    file += "    }\n";
    file += "}\n";
    file += "#[derive(Debug)]\n";
    file += "pub enum Instruction {\n";
    for i in &instructions.instruction {
        if i.arg_types.is_empty() {
            file += format!("    {},\n", i.opname).as_str();
        } else {
            file += format!("    {}({}),\n", i.opname, iter::repeat("Arg").take(i.arg_types.len()).collect::<Vec<&str>>().join(", ")).as_str();
        }
    }
    file += "}\n";

    file += "impl Instruction {\n";
    file += "    pub fn opcode(&self) -> u8 {\n";
    file += "        use Instruction::*;\n";
    file += "        match self {\n";
    for i in &instructions.instruction {
        if i.arg_types.is_empty() {
            file += format!("            {}", i.opname).as_str();
        } else {
            file += format!("            {}({})", i.opname, iter::repeat("_").take(i.arg_types.len()).collect::<Vec<&str>>().join(", ")).as_str();
        }
        file += format!(" => {:#x},\n", i.opcode).as_str();
    }
    file += "        }\n";
    file += "    }\n";
    file += "\n";

    file += "    pub fn args(&self) -> Vec<crate::code2::Arg> {\n";
    file += "        use Instruction::*;\n";
    file += "        match self {\n";
    for i in &instructions.instruction {
        if i.arg_types.is_empty() {
            file += format!("            {}", i.opname).as_str();
        } else {
            file += format!("            {}({})", i.opname, (1..).map(|i| format!("a{i}")).take(i.arg_types.len()).collect::<Vec<String>>().join(", ")).as_str();
        }
        file += format!(" => vec![{}],\n", (1..).map(|i| format!("a{i}.clone()")).take(i.arg_types.len()).collect::<Vec<String>>().join(", ")).as_str();
    }
    file += "        }\n";
    file += "    }\n";

    file += "\n";
    file += "    pub fn addressing_mode(&self) -> AddressingMode {\n";
    file += "        let args = self.args();\n";
    file += "        let padded_args = match args.len() {\n";
    file += "            0 => args,\n";
    file += "            1..=4 => [[0].repeat(4 - args.len()).iter().map(|z| Arg::Stack(0)).collect(), args].concat(),\n";
    file += "            5..=8 => [[0].repeat(8 - args.len()).iter().map(|z| Arg::Stack(0)).collect(), args].concat(),\n";
    file += "            _ => panic!(\"Args length should be less than or equal to 8\"),\n";
    file += "        };\n";
    file += "        if padded_args.len() == 4 {\n";
    file += "            let mut m1: u8 = 0;\n";
    file += "            for i in 0..4 {\n";
    file += "                m1 <<= 2;\n";
    file += "                m1 |= modifier_bits(&padded_args[i]);\n";
    file += "            }\n";
    file += "            AddressingMode::Short(m1)\n";
    file += "        } else if padded_args.len() == 8 {\n";
    file += "            let mut m1: u8 = 0;\n";
    file += "            for i in 0..4 {\n";
    file += "                m1 <<= 2;\n";
    file += "                m1 |= modifier_bits(&padded_args[i]);\n";
    file += "            }\n";
    file += "            let mut m2: u8 = 0;\n";
    file += "            for i in 4..8 {\n";
    file += "                m2 <<= 2;\n";
    file += "                m2 |= modifier_bits(&padded_args[i]);\n";
    file += "            }\n";
    file += "            AddressingMode::Long { low: m1, high: m2 }\n";
    file += "        } else {\n";
    file += "            panic!(\"Args were not padded correctly\")\n";
    file += "        }\n";
    file += "    }\n";

    file += "}\n";

    file += "fn modifier_bits(arg: &crate::code2::Arg) -> u8 {\n";
    file += "    match arg {\n";
    file += "        Arg::Stack(_) => 0b00,\n";
    file += "        Arg::Arg(_) => 0b01,\n";
    file += "        Arg::Var(_) => 0b10,\n";
    file += "        Arg::Immediate(_) => 0b11,\n";
    file += "    }\n";
    file += "}\n";

    std::fs::write("src/instruction.rs", file)
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
        generate_instructions_enum()
    }
}
