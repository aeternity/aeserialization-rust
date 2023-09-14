use std::{collections::BTreeMap, str, vec};

use aeser::{
    rlp::{RlpItem, ToRlpItem},
    Bytes,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::{
    data::{
        error::{DeserErr, SerErr},
        types::Type,
        value::Value,
    },
    instruction::{AddressingMode, Instruction},
};

pub trait Serializable {
    fn serialize(&self) -> Result<Bytes, SerErr>;
}

pub trait Deserializable: Sized {
    fn deserialize(bytes: &Bytes) -> Result<Self, DeserErr> {
        let (deser, rest) = Self::try_deserialize(bytes)?;
        if rest.is_empty() {
            Err(DeserErr::Failed)
        } else {
            Ok(deser)
        }
    }

    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr>;
}

impl Serializable for Contract {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let ser = [
            self.code.serialize()?.to_rlp_item().serialize(),
            self.symbols.serialize()?.to_rlp_item().serialize(),
            self.annotations.serialize()?.to_rlp_item().serialize(),
        ]
        .concat();
        Ok(ser)
    }
}

impl Deserializable for Contract {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let (rlp_code, rest1) =
            RlpItem::try_deserialize(bytes).map_err(|_| DeserErr::BadRlpItem)?;
        let (rlp_symbols, rest2) =
            RlpItem::try_deserialize(rest1).map_err(|_| DeserErr::BadRlpItem)?;
        let rlp_annotations = RlpItem::deserialize(rest2).map_err(|_| DeserErr::BadRlpItem)?;

        let code_bytes = rlp_code.byte_array().map_err(|_| DeserErr::BadRlpItem)?;
        let symbols_bytes = rlp_symbols.byte_array().map_err(|_| DeserErr::BadRlpItem)?;
        let annotations_bytes = rlp_annotations
            .byte_array()
            .map_err(|_| DeserErr::BadRlpItem)?;

        let code = Vec::<Function>::deserialize(&code_bytes)?;
        let symbols = Symbols::deserialize(&symbols_bytes)?;
        let annotations = Vec::<Annotation>::deserialize(&annotations_bytes)?;

        Ok((
            Contract {
                code,
                symbols,
                annotations,
            },
            &[],
        ))
    }
}

impl Serializable for Vec<Function> {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let mut map = BTreeMap::new();
        for fun in self {
            if map.insert(fun.id.serialize()?, fun) != None {
                Err(SerErr::DuplicateFunctionName)?;
            }
        }

        let mut ser = Vec::new();
        for fun in map.values() {
            ser.extend(fun.serialize()?);
        }
        Ok(ser)
    }
}

impl Deserializable for Vec<Function> {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let mut funs = vec![];
        loop {
            let (fun, rest) = Function::try_deserialize(bytes)?;
            funs.push(fun);
            if rest.is_empty() {
                break;
            }
        }
        Ok((funs, &[]))
    }
}

impl Serializable for Symbols {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let fate_vals_map = self
            .symbols
            .iter()
            .map(|(k, v)| {
                (
                    Value::String(k.to_vec()),
                    Value::String(v.as_bytes().to_vec()),
                )
            })
            .collect();
        Ok(Value::Map(fate_vals_map).serialize()?)
    }
}

impl Deserializable for Symbols {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let mut symbols = BTreeMap::new();
        match Value::deserialize(bytes)? {
            Value::Map(map) => {
                for (key, val) in map.iter() {
                    match (key, val) {
                        (Value::String(k), Value::String(v)) => {
                            symbols.insert(
                                k.to_vec(),
                                str::from_utf8(v)
                                    .map_err(|_| DeserErr::BadString)
                                    .map(|s| s.to_string())?,
                            );
                        }
                        _ => Err(DeserErr::BadSymbolsTable)?,
                    }
                }
            }
            _ => Err(DeserErr::BadSymbolsTable)?,
        }
        Ok((Symbols { symbols }, &[]))
    }
}

impl Serializable for Vec<Annotation> {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let mut map = BTreeMap::new();
        for ann in self {
            match ann {
                Annotation::Comment { line, comment } => {
                    let key = Value::Tuple(vec![
                        Value::String("comment".as_bytes().to_vec()),
                        Value::Integer(BigInt::from(*line)),
                    ]);
                    let val = Value::String(comment.as_bytes().to_vec());
                    map.insert(key, val);
                }
            }
        }
        Ok(Value::Map(map).serialize()?)
    }
}

impl Deserializable for Vec<Annotation> {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let mut anns = vec![];
        match Value::deserialize(bytes)? {
            Value::Map(map) => {
                for (key, val) in map.iter() {
                    match (key, val) {
                        (Value::Tuple(tag_line), Value::String(comment)) => match &tag_line[..] {
                            [Value::String(_), Value::Integer(big_line)] => {
                                anns.push(Annotation::Comment {
                                    line: big_line.to_u32().ok_or(DeserErr::BadAnnotation)?,
                                    comment: str::from_utf8(comment)
                                        .map(|s| s.to_string())
                                        .map_err(|_| DeserErr::BadAnnotation)?,
                                })
                            }
                            _ => Err(DeserErr::BadAnnotation)?,
                        },
                        _ => Err(DeserErr::BadAnnotation)?,
                    }
                }
            }
            _ => Err(DeserErr::BadAnnotation)?,
        }
        Ok((anns, &[]))
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
        ]
        .concat();
        Ok(ser)
    }
}

impl Deserializable for Function {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        unimplemented!()
    }
}

impl Serializable for Attributes {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        Ok(vec![*self as u8])
    }
}

impl Deserializable for Attributes {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let attr = match bytes[..] {
            [0] => Attributes::None,
            [1] => Attributes::Private,
            [2] => Attributes::Payable,
            [3] => Attributes::PrivatePayable,
            _ => Err(DeserErr::BadAttributes)?,
        };
        Ok((attr, &[]))
    }
}

impl Serializable for TypeSig {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        Ok([
            Type::Tuple(self.args.clone()).serialize()?,
            self.ret.serialize()?,
        ]
        .concat())
    }
}

impl Deserializable for TypeSig {
    fn try_deserialize(bytes: &Bytes) -> Result<(Self, &[u8]), DeserErr> {
        let (args_tuple, ret_rest) = Type::deserialize(bytes)?;
        let (ret, rest) = Type::deserialize(ret_rest)?;
        match args_tuple {
            Type::Tuple(args) => Ok((TypeSig { args, ret }, rest)),
            _ => Err(DeserErr::BadTypeSig),
        }
    }
}

impl Serializable for Instruction {
    fn serialize(&self) -> Result<Bytes, SerErr> {
        let ser = [
            vec![self.opcode()],
            self.addressing_mode().serialize()?,
            self.args().serialize()?,
        ]
        .concat();
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

impl Serializable for Vec<Vec<Instruction>> {
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
            Arg::Stack(n) | Arg::Arg(n) | Arg::Var(n) => {
                Value::Integer(BigInt::from(*n)).serialize()
            }
            Arg::Immediate(v) => v.serialize(),
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

#[derive(Debug, PartialEq)]
pub struct Contract {
    pub code: Vec<Function>,
    pub symbols: Symbols,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, PartialEq)]
pub struct Code {
    // TODO: no need to store as map? map is only needed for sorting?
    functions: BTreeMap<Bytes, Function>,
}

#[derive(Debug, PartialEq)]
pub struct Symbols {
    symbols: BTreeMap<Bytes, String>,
}

#[derive(Debug, PartialEq)]
pub enum Annotation {
    Comment { line: u32, comment: String },
}

#[derive(Debug, PartialEq)]
pub struct Id {
    id_str: String,
}

impl Id {
    pub fn new(id_str: String) -> Self {
        Id { id_str }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub id: Id,
    pub attributes: Attributes,
    pub type_sig: TypeSig,
    pub instructions: Vec<Vec<Instruction>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attributes {
    None = 0,
    Private = 1,
    Payable = 2,
    PrivatePayable = 3,
}

#[derive(Debug, PartialEq)]
pub struct TypeSig {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    Stack(u32),
    Arg(u32),
    Var(u32),
    Immediate(Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use aeser::rlp::ToRlpItem;
    use num_bigint::BigInt;
    use proptest::prelude::*;

    fn arb_function() -> impl Strategy<Value = Function> {
        any::<u32>().prop_map(|_x| Function {
            id: Id {
                id_str: String::from("str"),
            },
            attributes: Attributes::None,
            type_sig: TypeSig {
                args: vec![],
                ret: Type::Address,
            },
            instructions: vec![],
        })
    }

    fn arb_id() -> impl Strategy<Value = Id> {
        any::<String>().prop_map(|s| Id { id_str: s })
    }

    fn arb_symbols() -> impl Strategy<Value = Symbols> {
        any::<u32>().prop_map(|_x| Symbols {
            symbols: BTreeMap::new(),
        })
    }

    fn arb_attrs() -> impl Strategy<Value = Attributes> {
        any::<u32>().prop_map(|_x| Attributes::None)
    }

    fn arb_arg() -> impl Strategy<Value = Arg> {
        any::<u32>().prop_map(|_x| Arg::Stack(0))
    }

    fn arb_annotation() -> impl Strategy<Value = Annotation> {
        any::<u32>().prop_map(|_x| Annotation::Comment {
            line: 1,
            comment: String::from("()"),
        })
    }

    fn arb_instruction() -> impl Strategy<Value = Instruction> {
        any::<u32>().prop_map(|_x| Instruction::Return)
    }

    fn arb_typesig() -> impl Strategy<Value = TypeSig> {
        any::<u32>().prop_map(|_x| TypeSig {
            args: vec![],
            ret: Type::Address,
        })
    }

    fn arb_contract() -> impl Strategy<Value = Contract> {
        (
            any::<Vec<Function>>(),
            arb_symbols(),
            any::<Vec<Annotation>>(),
        )
            .prop_map(|(code, symbols, annotations)| Contract {
                code,
                symbols,
                annotations,
            })
    }

    impl Arbitrary for Contract {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_contract().boxed()
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

    impl Arbitrary for Annotation {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_annotation().boxed()
        }
    }

    // Property Tests
    proptest! {
        #[test]
        fn test_contract_serialization_props(c: Contract) {
            let rlp_code = c.code.serialize().unwrap().to_rlp_item().serialize();
            let rlp_symbols = c.symbols.serialize().unwrap().to_rlp_item().serialize();
            let rlp_annotations = c.annotations.serialize().unwrap().to_rlp_item().serialize();
            prop_assert_eq!(c.serialize().unwrap(), [rlp_code, rlp_symbols, rlp_annotations].concat());
        }

        //#[test]
        //fn test_code_serialization_props(c: Vec<Function>) {
        //    let mut ser_funs = Vec::new();
        //    let names: Vec<Bytes> = c.functions.keys().cloned().collect();
        //    prop_assert!(names.windows(2).all(|w| w[0] <= w[1]));
        //    for fun in c.functions.values() {
        //        ser_funs.extend(fun.serialize().unwrap());
        //    }
        //    prop_assert_eq!(c.serialize().unwrap(), ser_funs);
        //}

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

    // Unit Tests
    #[test]
    fn test_init_id_serialization() {
        let id = Id {
            id_str: String::from("init"),
        };
        assert_eq!(id.serialize().unwrap(), vec![0x44, 0xd6, 0x44, 0x1f]);
    }

    #[test]
    fn test_serialize_contract() {
        let byte_code = vec![
            169, 254, 89, 123, 141, 76, 0, 55, 0, 103, 7, 103, 119, 23, 1, 3, 47, 3, 2, 47, 2, 13,
            98, 97, 114, 127, 13, 102, 111, 111, 255, 4, 47, 0, 6, 47, 1, 13, 102, 111, 111, 127,
            139, 47, 1, 17, 89, 123, 141, 76, 13, 109, 97, 112, 159, 47, 1, 43, 29, 99, 111, 109,
            109, 101, 110, 116, 2, 73, 32, 67, 79, 78, 84, 82, 65, 67, 84, 32, 109, 97, 112, 111,
            102, 109, 97, 112,
        ];

        let mut map1 = BTreeMap::new();
        map1.insert(
            Value::String("foo".as_bytes().to_vec()),
            Value::Boolean(true),
        );
        map1.insert(
            Value::String("bar".as_bytes().to_vec()),
            Value::Boolean(false),
        );
        let map2 = BTreeMap::new();
        let mut map3 = BTreeMap::new();
        map3.insert(
            Value::String("foo".as_bytes().to_vec()),
            Value::Boolean(false),
        );
        let mut map = BTreeMap::new();
        map.insert(Value::Integer(BigInt::from(1)), Value::Map(map1));
        map.insert(Value::Integer(BigInt::from(2)), Value::Map(map2));
        map.insert(Value::Integer(BigInt::from(3)), Value::Map(map3));
        let fun = Function {
            id: Id::new(String::from("map")),
            attributes: Attributes::None,
            type_sig: TypeSig {
                args: vec![],
                ret: Type::Map {
                    key: Box::new(Type::Integer),
                    val: Box::new(Type::Map {
                        key: Box::new(Type::String),
                        val: Box::new(Type::Boolean),
                    }),
                },
            },
            instructions: vec![vec![Instruction::Returnr(Arg::Immediate(Value::Map(map)))]],
        };

        let fun_name = "map";
        let fun_id = Id::new(String::from(fun_name)).serialize().unwrap();
        let mut map_symbols = BTreeMap::new();
        map_symbols.insert(fun_id, fun_name.to_string());

        let code = vec![fun];
        let symbols = Symbols {
            symbols: map_symbols,
        };
        let annotations = vec![Annotation::Comment {
            line: 1,
            comment: String::from(" CONTRACT mapofmap"),
        }];

        let contract = Contract {
            code,
            symbols,
            annotations,
        };

        assert_eq!(contract.serialize().unwrap(), byte_code);

        assert_eq!(Contract::deserialize(&byte_code).unwrap(), contract);
    }
}
