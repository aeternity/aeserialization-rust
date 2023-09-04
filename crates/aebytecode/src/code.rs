use std::{collections::BTreeMap, vec};

use aeser::{Bytes, rlp::ToRlpItem};
use num_bigint::BigInt;

use crate::{data::{types::Type, value::Value, error::SerErr}, instruction::{Instruction, AddressingMode}};

pub trait Serializable {
    fn serialize(&self) -> Result<Bytes, SerErr>;
}

impl Serializable for Contract {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let ser = [
            self.code.serialize()?.to_rlp_item().serialize(),
            self.symbols.serialize()?.to_rlp_item().serialize(),
            self.annotations.serialize()?.to_rlp_item().serialize(),
        ].concat();
        Ok(ser)
    }
}
impl Serializable for Code {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let mut ser = Vec::new();
        for fun in self.functions.values() {
            ser.extend(fun.serialize()?);
        }
        Ok(ser)
    }
}
impl Serializable for Symbols {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let fate_vals_map = self.symbols
            .iter()
            .map(|(k,v)|
                (Value::String(k.as_bytes().to_vec()), Value::Integer(BigInt::from(*v)))
            )
            .collect();
        Ok(Value::Map(fate_vals_map).serialize()?)
    }
}
impl Serializable for Annotations {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        if !self.annotations.is_empty() {
            panic!("Annotations are not an empty")
        }
        Ok(vec![])
    }
}
impl Serializable for Id {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        use blake2::{digest::consts::U32, Blake2b, Digest};
        type Blake2b32 = Blake2b<U32>;
        let mut hasher = Blake2b32::new();
        hasher.update(self.id_str.as_str());
        Ok(hasher.finalize()[0..4].to_vec())
    }
}
impl Serializable for Function {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let ser = [
            vec![0xfe],
            self.id.serialize()?,
            self.attributes.serialize()?,
            self.type_sig.serialize()?,
            self.instructions.serialize()?,
        ].concat();
        Ok(ser)
    }
}
impl Serializable for Attributes {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        Ok(vec![*self as u8])
    }
}
impl Serializable for TypeSig {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        Ok([Type::Tuple(self.args.clone()).serialize()?, self.ret.serialize()?].concat())
    }
}
impl Serializable for Instruction {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let ser = [
            vec![self.opcode()],
            self.addressing_mode().serialize()?,
            self.args().serialize()?,
        ].concat();
        Ok(ser)
    }
}
impl Serializable for Vec<Instruction> {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let mut ser = Vec::new();
        for instr in self {
            ser.extend(instr.serialize()?);
        }
        Ok(ser)
    }
}
impl Serializable for Arg {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        match self {
            Arg::Stack(n) | Arg::Arg(n) | Arg::Var(n) =>
                Value::Integer(BigInt::from(*n)).serialize(),
            Arg::Immediate(v) =>
                v.serialize(),
        }
    }
}
impl Serializable for Vec<Arg> {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let mut ser = Vec::new();
        for arg in self {
            ser.extend(arg.serialize()?)
        }
        Ok(ser)
    }
}
impl Serializable for AddressingMode {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        match self {
            Self::Short(low) => Ok(vec![*low]),
            Self::Long { high, low } => Ok(vec![*low, *high]),
        }
    }
}

#[derive(Debug)]
struct Contract {
    code: Code,
    symbols: Symbols,
    annotations: Annotations,
}

#[derive(Debug)]
struct Code {
    functions: BTreeMap<u32, Function>,
}

#[derive(Debug)]
struct Symbols {
    symbols: BTreeMap<String, u32>,
}

#[derive(Debug)]
struct Annotations {
    annotations: BTreeMap<u32, u32>,
}

#[derive(Debug)]
pub struct Id {
    id_str: String,
}

impl Id {
    pub fn new(id_str: String) -> Self {
        Id {
            id_str
        }
    }
}

#[derive(Debug)]
struct Function {
    id: Id,
    attributes: Attributes,
    type_sig: TypeSig,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Attributes {
    None = 0,
    Private = 1,
    Payable = 2,
    PrivatePayable = 3,
}

#[derive(Debug)]
struct TypeSig {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Stack(u32),
    Arg(u32),
    Var(u32),
    Immediate(Value),
}

#[cfg(test)]
mod test {
    use super::*;
    use aeser::rlp::ToRlpItem;
    use proptest::prelude::*;
    use num_bigint::BigInt;

    fn arb_function() -> impl Strategy<Value = Function> {
        any::<u32>()
            .prop_map(|_x|
                Function {
                    id: Id { id_str: String::from("str") },
                    attributes: Attributes::None,
                    type_sig: TypeSig { args: vec![], ret: Type::Address },
                    instructions: vec![],
                }
            )
    }

    fn arb_id() -> impl Strategy<Value = Id> {
        any::<String>()
            .prop_map(|s|
                Id { id_str: s }
            )
    }

    fn arb_code() -> impl Strategy<Value = Code> {
        any::<u32>()
            .prop_map(|_x|
                Code {
                    functions: BTreeMap::new(),
                }
            )
    }

    fn arb_symbols() -> impl Strategy<Value = Symbols> {
        any::<u32>()
            .prop_map(|_x|
                Symbols {
                    symbols: BTreeMap::new(),
                }
            )
    }

    fn arb_attrs() -> impl Strategy<Value = Attributes> {
        any::<u32>()
            .prop_map(|_x|
                Attributes::None
            )
    }

    fn arb_arg() -> impl Strategy<Value = Arg> {
        any::<u32>()
            .prop_map(|_x|
                Arg::Stack(0)
            )
    }

    fn arb_annotations() -> impl Strategy<Value = Annotations> {
        any::<u32>()
            .prop_map(|_x|
                Annotations {
                    annotations: BTreeMap::new(),
                }
            )
    }

    fn arb_instruction() -> impl Strategy<Value =Instruction> {
        any::<u32>()
            .prop_map(|_x|
                Instruction::ADDRESS
            )
    }

    fn arb_typesig() -> impl Strategy<Value = TypeSig> {
        any::<u32>()
            .prop_map(|_x|
                TypeSig {
                    args: vec![],
                    ret: Type::Address,
                }
            )
    }

    fn arb_contract() -> impl Strategy<Value = Contract> {
        (arb_code(), arb_symbols(), arb_annotations())
            .prop_map(|(code, symbols, annotations)|
                Contract {
                    code,
                    symbols,
                    annotations,
                }
            )
    }

    impl Arbitrary for Contract {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_contract().boxed()
        }
    }

    impl Arbitrary for Code {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_code().boxed()
        }
    }

    impl Arbitrary for Function {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_function().boxed()
        }
    }

    impl Arbitrary for Id {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_id().boxed()
        }
    }

    impl Arbitrary for Attributes {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_attrs().boxed()
        }
    }

    impl Arbitrary for TypeSig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_typesig().boxed()
        }
    }

    impl Arbitrary for Arg {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_arg().boxed()
        }
    }

    impl Arbitrary for Instruction {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_instruction().boxed()
        }
    }

    proptest! {
        #[test]
        fn test_contract_serialization_props(c: Contract) {
            let rlp_code = c.code.serialize().unwrap().to_rlp_item().serialize();
            let rlp_symbols = c.symbols.serialize().unwrap().to_rlp_item().serialize();
            let rlp_annotations = c.annotations.serialize().unwrap().to_rlp_item().serialize();
            prop_assert_eq!(c.serialize().unwrap(), [rlp_code, rlp_symbols, rlp_annotations].concat());
        }

        #[test]
        fn test_code_serialization_props(c: Code) {
            let mut ser_funs = Vec::new();
            let names: Vec<u32> = c.functions.keys().cloned().collect();
            prop_assert!(names.windows(2).all(|w| w[0] <= w[1]));
            for fun in c.functions.values() {
                ser_funs.extend(fun.serialize().unwrap());
            }
            prop_assert_eq!(c.serialize().unwrap(), ser_funs);
        }

        #[test]
        fn test_function_serialization_props(f: Function) {
            let ser_fun = [
                vec![0xfe],
                f.id.serialize().unwrap(),
                f.attributes.serialize().unwrap(),
                f.type_sig.serialize().unwrap(),
                f.instructions.serialize().unwrap(),
            ].concat();
            prop_assert_eq!(f.serialize().unwrap(), ser_fun);
        }

        #[test]
        fn test_id_serialization_props(id: Id) {
            prop_assert_eq!(id.serialize().unwrap().len(), 4);
        }

        #[test]
        fn test_attributes_serialization_props(attrs: Attributes) {
            let ser = match attrs {
                Attributes::None => 0,
                Attributes::Private => 1,
                Attributes::Payable => 2,
                Attributes::PrivatePayable => 3,
            };
            prop_assert_eq!(attrs.serialize().unwrap(), vec![ser]);
        }

        #[test]
        fn test_typesig_serialization_props(type_sig: TypeSig) {
            let ser_sig = [
                Type::Tuple(type_sig.args.to_vec()).serialize().unwrap(),
                type_sig.ret.serialize().unwrap(),
            ].concat();
            prop_assert_eq!(type_sig.serialize().unwrap(), ser_sig);
        }

        #[test]
        fn test_instructions_serialization_props(instructions: Vec<Instruction>) {
            let mut ser_instructions = Vec::new();
            for op in &instructions {
                ser_instructions.extend(op.serialize().unwrap());
            }
            prop_assert_eq!(instructions.serialize().unwrap(), ser_instructions);
        }

        #[test]
        fn test_instruction_serialization_props(instruction: Instruction) {
            let ser_instruction = [
                vec![instruction.opcode()],
                instruction.addressing_mode().serialize().unwrap(),
                instruction.args().serialize().unwrap(),
            ].concat();
            prop_assert_eq!(instruction.serialize().unwrap(), ser_instruction);
        }

        #[test]
        fn test_arguments_serialization_props(args: Vec<Arg>) {
            let mut ser_arguments = Vec::new();
            for arg in &args {
                ser_arguments.extend(arg.serialize().unwrap());
            }
            prop_assert_eq!(args.serialize().unwrap(), ser_arguments);
        }

        #[test]
        fn test_argument_serialization_props(arg: Arg) {
            let ser_arg = match &arg {
                Arg::Stack(n) | Arg::Arg(n) | Arg::Var(n) =>
                    Value::Integer(BigInt::from(*n)).serialize(),
                Arg::Immediate(d) =>
                    d.serialize(),
            };
            prop_assert_eq!(arg.serialize().unwrap(), ser_arg.unwrap());
        }
    }

    #[test]
    fn test_init_id_serialization() {
        let id = Id { id_str: String::from("init") };
        assert_eq!(id.serialize().unwrap(), vec![0x44, 0xd6, 0x44, 0x1f]);
    }
}
