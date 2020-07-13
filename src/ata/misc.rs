/*!
Functions implementing typical ATA commands.

## Example

```
use hdd::Device;
use hdd::ata::ATADevice;
use hdd::ata::misc::Misc;
use hdd::ata::data::id::Ternary;

...

// it is a good idea to get feature status with device id info before proceeding further
// the good thing is, ATA IDENTIFY DEVICE covers a lot of features, so we only need to call this once
let id = dev.get_device_id().unwrap();

match id.smart {
	Ternary::Unsupported => println!("SMART is not supported"),
	Ternary::Disabled => println!("SMART is disabled"),
	Ternary::Enabled => {
		let status = dev.get_smart_health().unwrap();
		println!("SMART health status: {}", match status {
			Some(true) => "good",
			Some(false) => "BAD",
			None => "(unknown)",
		});
	},
}
```
*/

#[cfg(not(target_os = "linux"))]
use Device;

use scsi;

use std::io;

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		IO(err: io::Error) {
			from()
			//from(ATAError::IO(err): ATAError) -> (err)
			display("IO error: {}", err)
			description(err.description())
			cause(err)
		}
		SCSI(err: scsi::ATAError) {
			from()
			display("{}", err)
		}
	}
}
