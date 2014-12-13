extern crate gossip;
use gossip::parse;

#[test]
fn foo() {
	let res = parse();
    assert_eq!(5i,res);
}
