mod expr;
mod types;

pub use expr::SclangExpression;
pub use types::SclangType;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expr::RESERVED_KEYWORDS;

    use super::*;

    #[test]
    fn test_literal() {
        let r = SclangExpression::parse(&mut "1234567890").unwrap();
        assert_eq!(r, SclangExpression::Literal(1234567890));

        let contain_alpha = SclangExpression::parse(&mut "123s");
        assert!(contain_alpha.is_err());

        let ends_alpha = SclangExpression::parse(&mut "123123123s");
        assert!(ends_alpha.is_err());
    }

    #[test]
    fn test_boolean() {
        let tr = SclangExpression::parse(&mut "true").unwrap();
        assert_eq!(tr, SclangExpression::Boolean(true));

        let fa = SclangExpression::parse(&mut "false").unwrap();
        assert_eq!(fa, SclangExpression::Boolean(false));

        // let contain_alpha = SclangExpression::parse(&mut "true123123");
        // assert!(contain_alpha.is_err());

        let misspelled = SclangExpression::parse(&mut "treu");
        assert!(
            misspelled.is_err() || !matches!(&misspelled.unwrap(), SclangExpression::Boolean(true))
        );

        // let starts_alpha = SclangExpression::parse(&mut "strue");
        // assert!(starts_alpha.is_err());
    }

    #[test]
    fn test_var() {
        let x = SclangExpression::parse(&mut "x").unwrap();
        assert_eq!(x, SclangExpression::Var("x".to_string()));
        let y = SclangExpression::parse(&mut "x123").unwrap();
        assert_eq!(y, SclangExpression::Var("x123".to_string()));
        let underscore = SclangExpression::parse(&mut "some_variable").unwrap();
        assert_eq!(
            underscore,
            SclangExpression::Var("some_variable".to_string())
        );

        let wrong = SclangExpression::parse(&mut "1x");
        assert!(wrong.is_err());

        for mut r in RESERVED_KEYWORDS.into_iter() {
            // make sure that the keyword is not parsed as a variable
            if let Ok(expr) = SclangExpression::parse(&mut r) {
                assert!(!matches!(expr, SclangExpression::Var(_)))
            }
        }
    }

    #[test]
    fn test_let() {
        let input = "let x = 1; x";
        let x = input.parse::<SclangExpression>();
        match x {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
    }

    #[test]
    fn test_add() {
        let input = "1 + 2";
        let x = input.parse::<SclangExpression>();
        match &x {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(x.is_ok());
    }

    #[test]
    fn test_fun() {
        let input = "let f = fun(x: num) { let z = x; true }; f(1)";
        let x = input.parse::<SclangExpression>();
        match &x {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(x.is_ok());
    }

    #[test]
    fn test_call() {
        let input = "add(1)";
        let x = input.parse::<SclangExpression>();
        match &x {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(x.is_ok());
    }

    #[test]
    fn test_multiple_let() {
        let input = "let x = fun(x: number) { x + 1 };
        let y = x(1) + 2;
        let z = x(y) + y;
        let z2 = 1;
        z + z2"
            .parse::<SclangExpression>();
        match &input {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(input.is_ok());
    }

    #[test]
    fn test_record() {
        let input = "{x = 3, y=2, z=2}".parse::<SclangExpression>();
        match &input {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        // assert!(input.is_ok());
    }

    #[test]
    fn test_record_access() {
        let input = "y + r.a".parse::<SclangExpression>();
        match &input {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(input.is_ok());
    }

    #[test]
    fn test_with() {
        let input = "with {x = 3, y = 2} do {x + y}".parse::<SclangExpression>();
        match &input {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        assert!(input.is_ok());
    }

    #[test]
    fn test_from_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/subtype.sclang");
        println!("path: {0:?}", path);
        let r = SclangExpression::from_file(path);
        match &r {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            }
        }
        println!("r: {0:?}", r);
    }
}
