mod nondeterministic;
mod tokenize;
fn main() {
	let regex = nondeterministic::Regex::from_simple_expression("^(abc(cd)+)?c$")
		.expect("Regex compiling failed!");

	println!("{:?}", regex);

	dbg!(regex.test("c"));
	dbg!(regex.test("abc"));
	dbg!(regex.test("abcc"));
}
