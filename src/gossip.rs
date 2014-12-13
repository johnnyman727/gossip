pub fn execute_io() {

  // parse this giant buffer
  parse();

  // Pull out packets

  // Execute packets in thread

  // conglomerate results into a single buffer

  // send buffer back



}

fn parse() -> int {

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
