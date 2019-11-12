# shtola

Shtola is a library for generic file processing. It enables you to build your
own applications that function as static site generators! Here's an example:

``` rust
use shtola::{Shtola, Plugin, IR, ShFile};
use std::path::PathBuf;

fn plugin() -> Plugin {
	Box::new(|ir: IR| {
		// Let's create a hash where we store updated files...
		let update_hash: HashMap<PathBuf, ShFile> = HashMap::new();
		// ...get the file we want to update...
		let file = ir.files.get("current_time.txt".into()).unwrap();
		// ...update the file and add it to the update hash...
		update_hash.insert(
			"current_time.txt".into(),
			ShFile { content: "12:30".into(), ..file }
		);
		// ...and return the whole thing by calculating the union between our hashes!
		IR { files: update_hash.union(ir.files), ..ir }
	})
}

let mut s = Shtola::new();
s.source("my_source");
s.destination("my_destination")
s.use(plugin());
s.build().expect("Build failed!");
// Now we have a "current_time.txt" file in our destination directory that
// contains "12:30"!
```

## Installation

Add the latest version of Shtola to your `Cargo.toml`.

## Documentation

See https://docs.rs/shtola.

