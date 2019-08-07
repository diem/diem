use crate::substitution_map::SubstitutionMap;
use vm::file_format::{SignatureToken, StructHandleIndex};

#[test]
fn test_substitution() {
    let map = SubstitutionMap::new();
    {
        let map_1 = map
            .subst(vec![SignatureToken::Address, SignatureToken::U64])
            .unwrap();
        {
            let map_2 = map_1
                .subst(vec![
                    SignatureToken::TypeParameter(0),
                    SignatureToken::Struct(
                        StructHandleIndex::new(0),
                        vec![
                            SignatureToken::TypeParameter(0),
                            SignatureToken::TypeParameter(1),
                        ],
                    ),
                ])
                .unwrap();
            assert_eq!(map_2.materialize(0).unwrap(), SignatureToken::Address);
            assert_eq!(
                map_2.materialize(1).unwrap(),
                SignatureToken::Struct(
                    StructHandleIndex::new(0),
                    vec![SignatureToken::Address, SignatureToken::U64]
                )
            );
        }
    }
}
