use comrak::{markdown_to_html, ComrakOptions};
use shtola::{HashMap, Plugin, ShFile, IR};
use std::path::PathBuf;

pub fn plugin() -> Plugin {
	Box::new(|ir: IR| {
		let markdown_files = ir
			.files
			.iter()
			.filter(|(p, _)| p.extension().unwrap() == "md");
		let mut update_hash: HashMap<PathBuf, ShFile> = HashMap::new();
		for (path, file) in markdown_files {
			let mut p = path.clone();
			p.set_extension("html");
			update_hash.insert(
				p,
				ShFile {
					content: markdown_to_html(
						std::str::from_utf8(&file.content).unwrap(),
						&ComrakOptions::default(),
					)
					.into(),
					frontmatter: file.frontmatter.clone(),
				},
			);
		}
		IR {
			files: update_hash.union(ir.files),
			..ir
		}
	})
}
