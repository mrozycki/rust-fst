use fst::FST;
mod fst;

fn print_matches(fst: &FST, string: &str) {
    fst.match_string(string).into_iter()
        .for_each(|path| println!("{}", path.join("")));
}

fn main() {
    let digits = vec!('0', '1', '2', '3', '4', '5', '6', '7', '8', '9');
    let number = FST::at_least_once(FST::one_of_symbols(digits));
    let number_token = FST::wrap(number, "NUM");

    let operators = vec!('+', '-', '*', '/', '^');
    let operator = FST::one_of_symbols(operators);
    let operator_token = FST::wrap(operator, "OP");

    let open_paren_token = FST::wrap(FST::consume(FST::symbol('(')), "OPEN_PAREN");
    let close_paren_token = FST::wrap(FST::consume(FST::symbol(')')), "CLOSE_PAREN");

    let whitespace = FST::one_of_symbols(vec!(' ', '\n', '\t'));
    let consume_whitespace = FST::consume(whitespace);

    let lexer = FST::at_least_once(FST::one_of(vec!(
        number_token, operator_token, open_paren_token, close_paren_token, consume_whitespace
    )));
    println!("{}", lexer);
    print_matches(&lexer, "(81 + 27) * 13 + 42");
}
