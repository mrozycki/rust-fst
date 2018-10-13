use fst::FST;
mod fst;

fn print_matches(fst: &FST, string: &str) {
    fst.match_string(string).into_iter()
        .for_each(|path| println!("{}", path.join("")));
}

fn main() {
    let fst = FST::wrap(FST::string("abc"), "DUPA");
    println!("{}", fst);
    print_matches(&fst, "abc");
}
