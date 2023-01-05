mod nondeterministic;
fn main() {
	let regex =
		nondeterministic::Regex::from_simple_expression("a|b").expect("Regex compiling failed!");

	println!("{:?}", regex);

	dbg!(regex.test("a"));
	dbg!(regex.test("b"));
	dbg!(regex.test("c"));
}
