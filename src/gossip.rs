pub fn parse() -> int {
    println!("parsed");
    5i
}

#[cfg(test)]
mod test {
	use super::parse;
	#[test]
	fn test_parse() {
	    let result = parse();
	    assert_eq!(5i, result);
	}
}
