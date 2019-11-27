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
		let mut removal_hash: HashMap<PathBuf, ShFile> = HashMap::new();
		for (path, file) in markdown_files {
			let mut p = path.clone();
			p.set_extension("html");
			removal_hash.insert(path.to_path_buf(), ShFile::empty());
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
			files: update_hash.union(ir.files).difference(removal_hash),
			..ir
		}
	})
}

#[test]
fn it_works() {
	use shtola::Shtola;

	let mut s = Shtola::new();
	s.source("../fixtures/markdown");
	s.destination("../fixtures/markdown/dest");
	s.clean(true);
	s.register(plugin());
	let r = s.build().unwrap();
	let file: &ShFile = r.files.get(&PathBuf::from("hello.html")).unwrap();
	assert_eq!(
		std::str::from_utf8(&file.content).unwrap(),
		"<h1>Hello!</h1>\n<p>What's going <em>on</em>?</p>\n"
	)
}
