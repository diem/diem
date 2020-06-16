use sgx_isa::{Report, Targetinfo};

pub fn sgx_call() {

	let targetinfo = Targetinfo::from(Report::for_self());
	println!("this is galled from sgx...{:#?}", targetinfo);
}
