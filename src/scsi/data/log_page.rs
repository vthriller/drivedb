/*!
Functions to parse and structs to represent SCSI log pages.

For more, see SPC-4, 7.3 Log parameters.

## Example

```
use hdd::scsi::data::log_page;

let (_sense, data) = dev.log_sense(...)?;

let page = log_page::parse(&data).unwrap();
println!("{:#?}", page);

let params = page.parse_params();
for param in params {
	println!("{:#?}", param);
}
```
*/

use byteorder::{ReadBytesExt, BigEndian};
