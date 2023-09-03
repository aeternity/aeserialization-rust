use std::{collections::BTreeMap, vec};

use aeser::Bytes;

use crate::{data::{types::Type, value::Value}, fate_op::FateOp};

trait Serializable {
    fn serialize(&self) -> Bytes {
        vec![]
    }
}

impl Serializable for Contract {}
impl Serializable for Code {}
impl Serializable for Symbols {}
impl Serializable for Annotations {}
impl Serializable for Id {}
impl Serializable for Function {}
impl Serializable for Attributes {}
impl Serializable for TypeSig {}
impl Serializable for Instructions {}
impl Serializable for FateOp {}
impl Serializable for Arguments {}
impl Serializable for Arg {}

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

}

#[derive(Debug)]
struct Annotations {

}

#[derive(Debug)]
struct Id {
    name: String,
}

#[derive(Debug)]
struct Function {
    id: Id,
    attributes: Attributes,
    type_sig: TypeSig,
    instructions: Instructions,
}

#[derive(Debug)]
struct Attributes {
    attrs: Vec<Attribute>,
}

#[derive(Debug, PartialEq)]
enum Attribute {
    Private = 1,
    Payable = 2,
}

#[derive(Debug)]
struct TypeSig {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug)]
struct Instructions {
    ops: Vec<FateOp>,
}

#[derive(Debug)]
pub struct Arguments {
    pub args: Vec<Arg>,
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

    fn arb_function() -> impl Strategy<Value = Function> {
        any::<u32>()
            .prop_map(|_x|
                Function {
                    id: Id { name: String::from("str") },
                    attributes: Attributes { attrs: vec![] },
                    type_sig: TypeSig { args: vec![], ret: Type::Address },
                    instructions: Instructions { ops: vec![] },
                }
            )
    }

    fn arb_id() -> impl Strategy<Value = Id> {
        any::<String>()
            .prop_map(|s|
                Id { name: s }
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
                }
            )
    }

    fn arb_attrs() -> impl Strategy<Value = Attributes> {
        any::<u32>()
            .prop_map(|_x|
                Attributes {
                    attrs: vec![]
                }
            )
    }

    fn arb_arguments() -> impl Strategy<Value = Arguments> {
        any::<u32>()
            .prop_map(|_x|
                Arguments {
                    args: vec![]
                }
            )
    }

    fn arb_annotations() -> impl Strategy<Value = Annotations> {
        any::<u32>()
            .prop_map(|_x|
                Annotations {
                }
            )
    }

    fn arb_instructions() -> impl Strategy<Value = Instructions> {
        any::<u32>()
            .prop_map(|_x|
                Instructions{
                    ops: vec![],
                }
            )
    }

    fn arb_instruction() -> impl Strategy<Value = FateOp> {
        any::<u32>()
            .prop_map(|_x|
                FateOp::ADDRESS
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

    impl Arbitrary for Instructions {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_instructions().boxed()
        }
    }

    impl Arbitrary for Arguments {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_arguments().boxed()
        }
    }

    impl Arbitrary for FateOp {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            arb_instruction().boxed()
        }
    }

    proptest! {
        #[test]
        fn test_contract_serialization_props(c: Contract) {
            let rlp_code = c.code.serialize().to_rlp_item().serialize();
            let rlp_symbols = c.symbols.serialize().to_rlp_item().serialize();
            let rlp_annotations = c.annotations.serialize().to_rlp_item().serialize();
            prop_assert_eq!(c.serialize(), [rlp_code, rlp_symbols, rlp_annotations].concat());
        }

        #[test]
        fn test_code_serialization_props(c: Code) {
            let mut ser_funs = Vec::new();
            let names: Vec<u32> = c.functions.keys().cloned().collect();
            prop_assert!(names.windows(2).all(|w| w[0] <= w[1]));
            for fun in c.functions.values() {
                ser_funs.extend(fun.serialize());
            }
            prop_assert_eq!(c.serialize(), ser_funs);
        }

        #[test]
        fn test_function_serialization_props(f: Function) {
            let ser_fun = [
                vec![0xfe],
                f.id.serialize(),
                f.attributes.serialize(),
                f.type_sig.serialize(),
                f.instructions.serialize(),
            ].concat();
            prop_assert_eq!(f.serialize(), ser_fun);
        }

        #[test]
        fn test_id_serialization_props(id: Id) {
            prop_assert_eq!(id.serialize().len(), 4);
        }

        #[test]
        fn test_attributes_serialization_props(attrs: Attributes) {
            let mut total = 0;
            if attrs.attrs.contains(&Attribute::Private) {
                total += 1;
            }
            if attrs.attrs.contains(&Attribute::Payable) {
                total += 2;
            }
            prop_assert_eq!(attrs.serialize(), vec![total]);
        }

        #[test]
        fn test_typesig_serialization_props(type_sig: TypeSig) {
            let ser_sig = [
                Type::Tuple(type_sig.args.to_vec()).serialize().unwrap(),
                type_sig.ret.serialize().unwrap(),
            ].concat();
            prop_assert_eq!(type_sig.serialize(), ser_sig);
        }

        #[test]
        fn test_instructions_serialization_props(instructions: Instructions) {
            let mut ser_instructions = Vec::new();
            for op in &instructions.ops {
                ser_instructions.extend(op.serialize());
            }
            prop_assert_eq!(instructions.serialize(), ser_instructions);
        }

        #[test]
        fn test_instruction_serialization_props(instruction: FateOp) {
            let ser_instruction = [
                vec![instruction.opcode()],
                //addressing_mode(instruction),
                instruction.args().serialize(),
            ].concat();
            prop_assert_eq!(instruction.serialize(), ser_instruction);
        }

        #[test]
        fn test_argument_serialization_props(arguments: Arguments) {
            let mut ser_arguments = Vec::new();
            for arg in &arguments.args {
                ser_arguments.extend(arg.serialize());
            }
            prop_assert_eq!(arguments.serialize(), ser_arguments);
        }
    }

    #[test]
    fn test_main_id_serialization() {
        let id = Id { name: String::from("main") };
        assert_eq!(id.serialize(), vec![0x44, 0xd6, 0x44, 0x1f]);
    }
}
