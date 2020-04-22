// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Error, Format, FormatHolder, Named, VariantFormat};

#[test]
fn test_format_visiting() {
    use Format::*;

    let format = ContainerFormat::Enum(
        vec![(
            0,
            Named {
                name: "foo".into(),
                value: VariantFormat::Tuple(vec![U8, U8, Seq(Box::new(U8))]),
            },
        )]
        .into_iter()
        .collect(),
    );
    let mut counter: usize = 0;
    format
        .visit(&mut |format| {
            match format {
                U8 => counter += 1,
                _ => (),
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(counter, 3);

    assert!(VariantFormat::Unknown.visit(&mut |_| Ok(())).is_err());
    assert!(Format::Unknown.visit(&mut |_| Ok(())).is_err());
}

#[test]
fn test_format_unification() {
    use Format::*;

    let mut x = Unknown;
    assert!(x.unify(U8).is_ok());
    assert_eq!(x, U8);
    assert_eq!(
        x.unify(U16).unwrap_err(),
        Error::Incompatible("U8".into(), "U16".into())
    );

    let mut x = Tuple(vec![Unknown, U32]);
    x.unify(Tuple(vec![U16, Unknown])).unwrap();
    assert_eq!(x, Tuple(vec![U16, U32]));

    for x in vec![
        Unknown,
        Unit,
        Bool,
        I8,
        I16,
        I32,
        I64,
        I128,
        U8,
        U16,
        U32,
        U64,
        U128,
        F32,
        F64,
        Char,
        Str,
        Bytes,
        TypeName("foo".into()),
        Option(Box::new(Unit)),
        Seq(Box::new(Unit)),
        Map {
            key: Box::new(Unit),
            value: Box::new(Unit),
        },
        Tuple(vec![Unknown]),
    ]
    .iter_mut()
    {
        assert!(x.unify(x.clone()).is_ok());
        assert_eq!(*x != Unknown, x.unify(TypeName("bar".into())).is_err());
        assert_eq!(*x != Unknown, x.unify(Option(Box::new(U32))).is_err());
        assert_eq!(*x != Unknown, x.unify(Seq(Box::new(U32))).is_err());
        assert_eq!(*x != Unknown, x.unify(Tuple(vec![])).is_err());
    }
}

#[test]
fn test_container_format_unification() {
    use ContainerFormat::*;
    use Format::*;

    let mut x = TupleStruct(vec![Unknown, U32]);
    x.unify(TupleStruct(vec![U16, Unknown])).unwrap();
    assert_eq!(x, TupleStruct(vec![U16, U32]));

    let mut x = Enum(
        vec![(
            0,
            Named {
                name: "foo".into(),
                value: VariantFormat::Tuple(vec![Unknown]),
            },
        )]
        .into_iter()
        .collect(),
    );
    assert!(x
        .unify(Enum(
            vec![(
                0,
                Named {
                    name: "foo".into(),
                    value: VariantFormat::Unit,
                }
            )]
            .into_iter()
            .collect()
        ))
        .is_err());
    assert!(x
        .unify(Enum(
            vec![(
                0,
                Named {
                    name: "foo".into(),
                    value: VariantFormat::Tuple(vec![U8]),
                }
            )]
            .into_iter()
            .collect()
        ))
        .is_ok());
    // We don't check for name collisions in variants.
    assert!(x
        .unify(Enum(
            vec![(
                1,
                Named {
                    name: "foo".into(),
                    value: VariantFormat::Unknown
                }
            )]
            .into_iter()
            .collect()
        ))
        .is_ok());

    for x in vec![
        UnitStruct,
        NewTypeStruct(Box::new(Unit)),
        TupleStruct(vec![Unknown]),
        Struct(vec![Named {
            name: "foo".into(),
            value: Unknown,
        }]),
        Enum(
            vec![(
                0,
                Named {
                    name: "foo".into(),
                    value: VariantFormat::Unknown,
                },
            )]
            .into_iter()
            .collect(),
        ),
    ]
    .iter_mut()
    {
        assert!(x.unify(x.clone()).is_ok());
        assert!(x.unify(NewTypeStruct(Box::new(U8))).is_err());
        assert!(x.unify(TupleStruct(vec![])).is_err());
        assert!(x
            .unify(Struct(vec![Named {
                name: "bar".into(),
                value: Unknown
            }]))
            .is_err());
        assert!(x
            .unify(Enum(
                vec![(
                    0,
                    Named {
                        name: "bar".into(),
                        value: VariantFormat::Unknown
                    }
                )]
                .into_iter()
                .collect()
            ))
            .is_err());
    }
}
