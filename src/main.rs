mod nondeterministic;
fn main() {
	let regex =
		nondeterministic::Regex::from_simple_expression("a.+").expect("Regex compiling failed!");

	println!("{:?}", regex);

	dbg!(regex.test("a"));
	dbg!(regex.test("aab"));
	dbg!(regex.test("b"));
}
