#![cfg_attr(test, allow(unused))]
#![feature(once_cell)]

pub mod context;
pub mod executor;
pub mod lazy;
pub mod plan;
pub mod udf;

#[cfg(feature = "use-sql")]
pub mod sql;

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use policy_core::expr::{BinaryOp, Expr};

    use crate::plan::physical_expr::*;

    #[test]
    fn test_physical_expr_serde() {
        let phys_expr: Arc<dyn PhysicalExpr> = Arc::new(FilterExpr {
            input: Arc::new(ColumnExpr {
                name: "foo".into(),
                expr: Expr::Column("foo".into()),
                schema: None,
            }),
            by: Arc::new(BinaryOpExpr {
                left: Arc::new(ColumnExpr {
                    name: "foo".into(),
                    expr: Expr::Column("foo".into()),
                    schema: None,
                }),
                op: BinaryOp::Ge,
                right: Arc::new(LiteralExpr {
                    literal: Box::new(0i8),
                    expr: Expr::Literal(Box::new(0i8)),
                }),
                expr: Expr::Wildcard,
            }),
            expr: Expr::Wildcard,
        });

        let s = serde_json::to_string(&phys_expr).unwrap();
        assert_eq!(
            r#"{"physical_expr":"FilterExpr","input":{"physical_expr":"ColumnExpr","name":"foo","expr":{"Column":"foo"},"schema":null},"by":{"physical_expr":"BinaryOpExpr","left":{"physical_expr":"ColumnExpr","name":"foo","expr":{"Column":"foo"},"schema":null},"op":"Ge","right":{"physical_expr":"LiteralExpr","literal":{"primitive_data":"i8","value":0},"expr":{"Literal":{"primitive_data":"i8","value":0}}},"expr":"Wildcard"},"expr":"Wildcard"}"#,
            &s
        );

        let der = serde_json::from_str::<Arc<dyn PhysicalExpr>>(&s);
        assert!(der.is_ok());
    }
}
