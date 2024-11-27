use sclang::expr::SclangExpression;
pub fn main() {
    let mut input = "falses";
    let output = SclangExpression::parse(&mut input);
    // let output = num_parser.parse_next(&mut input);
    println!("output: {0:?}", output);
}
