mod nondeterministic;
fn main() {
    let regex = nondeterministic::Regex::from_simple_expression("fre*or+.n?ge")
        .expect("Regex compiling failed!");

    println!("{:?}", regex);

    println!("{}", regex.test("freeorage"));
    println!("{}", regex.test("frorange"));
    println!("{}", regex.test("freeoange"));
    println!("{}", regex.test("freeorannge"));
}
