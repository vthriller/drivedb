use hdd::ata::misc::Misc;

use hdd::ata::data::attr;
use hdd::drivedb;
use hdd::drivedb::vendor_attribute;

use clap::{
	Arg,
	ArgMatches,
	App,
	SubCommand,
};

use serde_json;
use serde_json::value::ToJson;

use super::{open_drivedb, when_smart_enabled, arg_json, arg_drivedb};

fn bool_to_flag(b: bool, c: char) -> char {
	if b { c } else { '-' }
}

// XXX only `pretty_attributes` clearly shows failing/failed attributes
fn print_attributes(values: Vec<attr::SmartAttribute>) {
	if values.is_empty() {
		print!("No S.M.A.R.T. attributes found.\n");
		return;
	}

	print!("S.M.A.R.T. attribute values:\n");
	print!(" ID name                     flags        value worst thresh fail raw\n");
	for val in values {
		// > The NAME … should not exceed 23 characters
		print!("{:3} {:.<24} {}{}{}{}{}{}{}    {}   {}    {} {} {}\n",
			val.id,
			val.name.as_ref().unwrap_or(&"?".to_string()),
			bool_to_flag(val.pre_fail, 'P'),
			bool_to_flag(!val.online, 'O'),
			bool_to_flag(val.performance, 'S'),
			bool_to_flag(val.error_rate, 'R'),
			bool_to_flag(val.event_count, 'C'),
			bool_to_flag(val.self_preserving, 'K'),
			if val.flags == 0 { "     ".to_string() }
				else { format!("+{:04x}", val.flags) },
			val.value.map(|v| format!("{:3}", v)).unwrap_or("---".to_string()),
			val.worst.map(|v| format!("{:3}", v)).unwrap_or("---".to_string()),
			val.thresh.map(|v| format!("{:3}", v)).unwrap_or("(?)".to_string()),
			match (val.value, val.worst, val.thresh) {
				(Some(v), _, Some(t)) if v <= t => "NOW ",
				(_, Some(w), Some(t)) if w <= t => "past",
				// either value/worst are part of the `val.row`,
				// or threshold is not available,
				// or value never was below the threshold
				_ => "-   ",
			},
			val.raw,
		);
	}
	// based on the output of 'smartctl -A -f brief' (part of 'smartctl -x')
	print!("                             ││││││\n");
	print!("                             │││││K auto-keep\n");
	print!("                             ││││C event count\n");
	print!("                             │││R error rate\n");
	print!("                             ││S speed/performance\n");
	print!("                             │O updated during off-line testing\n");
	print!("                             P prefailure warning\n");
}

pub fn subcommand() -> App<'static, 'static> {
	SubCommand::with_name("attrs")
		.about("Prints a list of S.M.A.R.T. attributes")
		.arg(arg_json())
		.arg(arg_drivedb())
		.arg(Arg::with_name("vendorattribute")
			.multiple(true)
			.short("v") // smartctl-like
			.long("vendorattribute") // smartctl-like
			.takes_value(true)
			.value_name("id,format[:byteorder][,name]")
			.help("set display option for vendor attribute 'id'")
		)
}

pub fn attrs<T: Misc + ?Sized>(
	dev: &T,
	args: &ArgMatches,
) {
	let id = dev.get_device_id().unwrap();

	let user_attributes = args.values_of("vendorattribute")
		.map(|attrs| attrs.collect())
		.unwrap_or(vec![])
		.into_iter()
		.map(|attr| vendor_attribute::parse(attr).ok()) // TODO Err(_)
		.filter(|x| x.is_some())
		.map(|x| x.unwrap())
		.collect();

	let drivedb = open_drivedb(args.value_of("drivedb"));
	let dbentry = drivedb.as_ref().map(|drivedb| drivedb::match_entry(
		&id,
		drivedb,
		user_attributes,
	));

	let use_json = args.is_present("json");

	when_smart_enabled(&id.smart, "attributes", || {
		let values = dev.get_smart_attributes(&dbentry).unwrap();

		if use_json {
			print!("{}\n", serde_json::to_string(&values.to_json().unwrap()).unwrap());
		} else {
			print_attributes(values);
		}
	});
}
