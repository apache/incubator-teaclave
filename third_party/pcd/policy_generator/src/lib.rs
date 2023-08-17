use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub policy_parser, "/grammar/policy_definition_language.rs");

#[cfg(test)]
mod test {
    use crate::policy_parser;

    #[test]
    fn pdl_can_parse_numbers() {
        let num_str = [
            "+1", "+1.", "+.1", "+0.1", "1", "1.", " .1", "0.1", "((0.))", "((+1.0))",
        ];
        let num_parser = policy_parser::NumParser::new();
        for num in num_str {
            assert!(num_parser.parse(num).is_ok(), "parse failed: `{num}`");
        }
    }

    #[test]
    fn pdl_can_parse_keyword() {
        let keywords = [
            "allow",
            "Allow",
            " aLLOW",
            "AlLoW    ",
            "denY ",
            "scheMe   ",
            "filTer  ",
            "   RoW",
        ];
        let keyword_parser = policy_parser::KeywordParser::new();

        for keyword in keywords {
            assert!(
                keyword_parser.parse(keyword).is_ok(),
                "parse failed: `{keyword}`"
            );
        }
    }

    #[test]
    fn pdl_can_parse_scheme() {
        let schemes = [
            "differential_privacy(1)",
            "Dp(2,4)",
            "k_anon(3)",
            "T_closeness(3.1415926535)",
            "L_diversity(-999)",
        ];
        let scheme_parser = policy_parser::SchemeParser::new();

        for scheme in schemes {
            assert!(
                scheme_parser.parse(scheme).is_ok(),
                "parse failed: `{scheme}`"
            );
        }
    }

    #[test]
    fn pdl_can_parse_attribute_list() {
        let list = "((attribute (foo: str, bar: i64, baz: f32, __test: bool, (_random_data_abcd777: String))))";

        assert!(policy_parser::AttributeListParser::new()
            .parse(list)
            .is_ok());
    }

    #[test]
    #[cfg(feature = "ast-serde")]
    fn pdl_can_serialize_policies() {
        let simple_policy = r#"
            FOO ATTRIBUTE(foo: i64, bar: f32, baz: u64):
            (
                Deny (foo, baz), Deny (foo), Allow(foo),
            )
        "#;
        let policy = policy_parser::PolicyParser::new().parse(simple_policy);

        assert!(policy.is_ok());

        let policy = serde_json::to_string(&policy.unwrap()).unwrap();

        assert_eq!(
            r#"{"name":"FOO","schema":[["foo","Int64"],["bar","Float32"],["baz","UInt64"]],"clause":[{"Deny":["foo","baz"]},{"Deny":["foo"]},{"Allow":{"attribute_list":["foo"],"scheme":[]}}]}"#,
            policy
        );
    }
}
