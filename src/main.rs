mod nondeterministic;
fn main() {
	let mut regex =
		nondeterministic::Regex::from_simple_expression("abc").expect("Regex compiling failed!");

	println!("{}", regex.test("defabcdef"));
}
