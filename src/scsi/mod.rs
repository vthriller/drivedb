/*!
All things SCSI.

* Use [`data` module](data/index.html) to parse various low-level structures found in SCSI command replies.
* Import traits from porcelain modules (like [`pages`](pages/index.html)) to do typical tasks without needing to compose commands and parse responses yourself.
  * You can also use [`module ata`](../ata/index.html) to issue ATA commands using ATA PASS-THROUGH.
*/

pub mod pages;

use std::io;

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		IO(err: io::Error) {
			from()
			display("IO error: {}", err)
			description(err.description())
			cause(err)
		}
		// this is for Sense::Fixed(FixedData::Invalid(_))
		// pun definitely intented at this point
		Nonsense {}
	}
}

// FIXME naming: this isn't about ATA-level error, this is error related to ATA PASS-THROUGH command
quick_error! {
	#[derive(Debug)]
	pub enum ATAError {
		SCSI(err: Error) {
			from()
			from(err: io::Error) -> (Error::IO(err))
			display("{}", err)
		}
		/// Device does not support ATA PASS-THROUGH command
		NotSupported {}
		// no non-deferred sense is available, or there's no descriptors for ATA registers to be found
		NoRegisters {}
	}
}
