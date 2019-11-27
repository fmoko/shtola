use shtola::{HashMap, ShFile, Shtola, IR};
use std::path::PathBuf;

fn main() {
	pretty_env_logger::init();
	let mut s = Shtola::new();
	s.source("fixtures/simple");
	s.destination("fixtures/dest_write");
	s.clean(true);
	let mw = Box::new(|ir: IR| {
		let mut update_hash: HashMap<PathBuf, ShFile> = HashMap::new();
		for (k, v) in &ir.files {
			update_hash.insert(
				k.into(),
				ShFile {
					frontmatter: v.frontmatter.clone(),
					content: "hello".into(),
				},
			);
		}
		IR {
			files: update_hash.union(ir.files),
			..ir
		}
	});
	s.register(mw);
	s.build().unwrap();
}
