pub mod expr;
pub use expr::SclangExpression;

#[cfg(test)]
mod tests {
    use expr::RESERVED_KEYWORDS;
    use winnow::PResult;

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
        assert!(misspelled.is_err() || !matches!(&misspelled.unwrap(), SclangExpression::Boolean(true)));

        // let starts_alpha = SclangExpression::parse(&mut "strue");
        // assert!(starts_alpha.is_err());
    }

    #[test]
    fn test_var() {
        let x = SclangExpression::parse(&mut "x").unwrap();
        assert_eq!(x, SclangExpression::Variable("x".to_string()));
        let y = SclangExpression::parse(&mut "x123").unwrap();
        assert_eq!(y, SclangExpression::Variable("x123".to_string()));
        let underscore = SclangExpression::parse(&mut "some_variable").unwrap();
        assert_eq!(underscore, SclangExpression::Variable("some_variable".to_string()));

        let wrong = SclangExpression::parse(&mut "1x");
        assert!(wrong.is_err());

        for mut r in RESERVED_KEYWORDS.into_iter(){
            // make sure that the keyword is not parsed as a variable
            if let Ok(expr) = SclangExpression::parse(&mut r) {
                assert!(!matches!(expr, SclangExpression::Variable(_)))
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
            },
        }
    }

    #[test]
    fn test_add() {
        let input = "1 + 2";
        let x = input.parse::<SclangExpression>();
        match x {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            },
        }
    }

    #[test]
    fn test_multiple_let() {
        let input = "let x = 1;
        let y = 2;
        x".parse::<SclangExpression>();
        match input {
            Ok(s) => println!("s: {0:?}", s),
            Err(e) => {
                println!("e: {}", e)
            },
        }
    }
}