/*!
Use this module to match hard drive and SMART values it returns against smartmontools database.

## Example

```
use drivedb::{
	Loader,
	Type,
	vendor_attribute,
};

# fn main() -> Result<(), Box<dyn std::error::Error>> {

let mut loader = Loader::new();
// look for version updated with `update-smart-drivedb(8)` first
loader.load("/var/lib/smartmontools/drivedb/drivedb.h")
	.or_else(|_| loader.load("/usr/share/smartmontools/drivedb.h"))?;
// `?` is optional though: if nothing can be loaded, loader will still provide dummy db for us

let db = loader.db()?;

// extra attribute definitions that user might give
let user_attributes = vec!["9,minutes"]
	.into_iter()
	.map(|attr| vendor_attribute::parse(attr).unwrap())
	.collect();

let model = "ST3000DM001-9YN166";
let firmware = "CC24";
let drivetype = Some(Type::HDD);

let meta = db.render_meta(&model, &firmware, drivetype, &user_attributes);

assert!(meta.warning.is_some());
assert!(meta.warning.unwrap().starts_with("A firmware update for this drive may be available"));

let attr = meta.render_attribute(9).unwrap();
assert_eq!(attr.id, Some(9));
assert_eq!(attr.name, Some("Power_On_Minutes".to_string()));
assert_eq!(attr.format, "min2hour".to_string());
assert_eq!(attr.byte_order, "543210".to_string());
assert_eq!(attr.drivetype, None);

# Ok(())
# }
```
*/

#![warn(
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unsafe_code,
	unstable_features,
	unused_import_braces,
	unused_qualifications,
)]

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate nom;
extern crate regex;

mod parser;
mod presets;
mod drivedb;
mod loader;
pub mod vendor_attribute;
pub use self::vendor_attribute::{Attribute, Type};
pub use self::drivedb::{DriveDB, DriveMeta};
pub use self::loader::{Loader, Error};
