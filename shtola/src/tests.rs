use crate::{Shtola, IR, HashMap, ShFile, Yaml};
use crate::json::json;
use std::path::PathBuf;
use std::fs;

#[test]
fn read_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/simple");
	s.destination("../fixtures/dest_read");
	let r = s.build().unwrap();
	assert_eq!(r.files.len(), 1);
	let keys: Vec<&PathBuf> = r.files.keys().collect();
	assert_eq!(keys[0].to_str().unwrap(), "hello.txt");
}

#[test]
fn clean_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/simple");
	s.destination("../fixtures/dest_clean");
	s.clean(true);
	fs::create_dir_all("../fixtures/dest_clean").unwrap();
	fs::write("../fixtures/dest_clean/blah.foo", "").unwrap();
	s.build().unwrap();
	let fpath = PathBuf::from("../fixtures/dest_clean/blah.foo");
	assert_eq!(fpath.exists(), false);
}

#[test]
fn write_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/simple");
	s.destination("../fixtures/dest_write");
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
	let dpath = PathBuf::from("../fixtures/dest_write/hello.txt");
	assert!(dpath.exists());
	let file = &fs::read(dpath).unwrap();
	let fstring = String::from_utf8_lossy(file);
	assert_eq!(fstring, "hello");
}

#[test]
fn frontmatter_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/frontmatter");
	s.destination("../fixtures/dest_matter1");
	s.clean(true);
	let r = s.build().unwrap();
	let (_, matter_file) = r.files.iter().last().unwrap();
	let frontmatter = matter_file.frontmatter[0]
		.as_hash()
		.unwrap()
		.get(&Yaml::from_str("hello"))
		.unwrap()
		.as_str()
		.unwrap();
	assert_eq!(frontmatter, "bro");
}

#[test]
fn no_frontmatter_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/frontmatter");
	s.destination("../fixtures/dest_matter2");
	s.clean(true);
	s.frontmatter(false);
	let r = s.build().unwrap();
	let (_, matter_file) = r.files.iter().last().unwrap();
	dbg!(matter_file);
	assert!(matter_file.frontmatter.is_empty());
}

#[test]
fn ignore_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/ignore");
	s.destination("../fixtures/dest_ignore");
	s.ignores(&mut vec!["ignored.md".to_string()]);
	s.clean(true);
	let r = s.build().unwrap();
	assert_eq!(r.files.len(), 1);
	let (path, _) = r.files.iter().last().unwrap();
	assert_eq!(path.to_str().unwrap(), "not_ignored.md");
}


#[test]
fn metadata_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/simple");
	s.destination("../fixtures/dest_meta");
	s.clean(true);
	let mw1 = Box::new(|ir: IR| {
		let metadata = ir.metadata.update("test".into(), json!("foo"))
															.update("test2".into(), json!({"bar": "baz"}));
		IR { metadata, ..ir }
	});

	let mw2 = Box::new(|ir: IR| {
		let metadata = ir.metadata.update("test".into(), json!(["a", "b", "c"]));
		IR { metadata, ..ir }
	});

	s.register(mw1);
	s.register(mw2);
	let r = s.build().unwrap();
	assert_eq!(r.metadata.get("test").unwrap(), &json!(["a", "b", "c"]));
	assert_eq!(r.metadata.get("test2").unwrap(), &json!({"bar": "baz"}));
}
