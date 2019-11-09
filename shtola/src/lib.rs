use globset::{Glob, GlobSetBuilder};
use pathdiff::diff_paths;
use std::default::Default;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

pub use im::HashMap;
pub use ware::Ware;
pub use yaml_rust::Yaml;

mod frontmatter;

pub struct Shtola {
	ware: Ware<IR>,
	ir: IR,
}

impl Shtola {
	pub fn new() -> Shtola {
		let config: Config = Default::default();
		let ir = IR {
			files: HashMap::new(),
			config,
		};
		Shtola {
			ware: Ware::new(),
			ir,
		}
	}

	pub fn ignores(&mut self, vec: &mut Vec<String>) {
		self.ir.config.ignores.append(vec);
		self.ir.config.ignores.dedup();
	}

	pub fn source<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.source = fs::canonicalize(path.into()).unwrap();
	}

	pub fn destination<T: Into<PathBuf> + Clone>(&mut self, path: T) {
		fs::create_dir_all(path.clone().into()).expect("Unable to create destination directory!");
		self.ir.config.destination = fs::canonicalize(path.into()).unwrap();
	}

	pub fn clean(&mut self, b: bool) {
		self.ir.config.clean = b;
	}

	pub fn frontmatter(&mut self, b: bool) {
		self.ir.config.frontmatter = b;
	}

	pub fn register(&mut self, func: Box<dyn Fn(IR) -> IR>) {
		self.ware.wrap(func);
	}

	pub fn build(&mut self) -> Result<IR, std::io::Error> {
		if self.ir.config.clean {
			fs::remove_dir_all(&self.ir.config.destination)?;
			fs::create_dir_all(&self.ir.config.destination)
				.expect("Unable to recreate destination directory!");
		}

		let mut builder = GlobSetBuilder::new();
		for item in &self.ir.config.ignores {
			builder.add(Glob::new(item).unwrap());
		}
		let set = builder.build().unwrap();
		let unfiltered_files = read_dir(&self.ir.config.source, self.ir.config.frontmatter)?;
		let files = unfiltered_files.iter().filter(|(p, _)| {
			let path = p.to_str().unwrap();
			!set.is_match(path)
		});

		self.ir.files = files.cloned().collect();
		let result_ir = self.ware.run(self.ir.clone());
		write_dir(result_ir.clone(), &self.ir.config.destination)?;
		Ok(result_ir)
	}
}

#[derive(Debug, Clone)]
pub struct IR {
	files: HashMap<PathBuf, ShFile>,
	config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
	ignores: Vec<String>,
	source: PathBuf,
	destination: PathBuf,
	clean: bool,
	frontmatter: bool,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			ignores: Vec::new(),
			source: PathBuf::from("."),
			destination: PathBuf::from("./dest"),
			clean: false,
			frontmatter: true,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ShFile {
	frontmatter: Vec<Yaml>,
	content: Vec<u8>,
}

fn read_dir(
	source: &PathBuf,
	frontmatter: bool,
) -> Result<HashMap<PathBuf, ShFile>, std::io::Error> {
	let mut result = HashMap::new();
	let iters = WalkDir::new(source)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|e| !e.path().is_dir());
	for entry in iters {
		let path = entry.path();
		let file: ShFile;
		let mut content = String::new();
		fs::File::open(path)?.read_to_string(&mut content)?;
		if frontmatter {
			let (matter, content) = frontmatter::lexer(&content);
			let yaml = frontmatter::to_yaml(&matter);
			file = ShFile {
				frontmatter: yaml,
				content: content.into(),
			};
		} else {
			file = ShFile {
				frontmatter: Vec::new(),
				content: content.into(),
			};
		}
		let rel_path = diff_paths(path, source).unwrap();
		result.insert(rel_path, file);
	}
	Ok(result)
}

fn write_dir(ir: IR, dest: &PathBuf) -> Result<(), std::io::Error> {
	for (path, file) in ir.files {
		let dest_path = dest.join(path);
		fs::create_dir_all(dest_path.parent().unwrap())
			.expect("Unable to create destination subdirectory!");
		fs::File::create(dest_path)?.write_all(&file.content)?;
	}
	Ok(())
}

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
	assert_eq!(
		matter_file.frontmatter[0]
			.as_hash()
			.unwrap()
			.get(&Yaml::from_str("hello"))
			.unwrap()
			.as_str()
			.unwrap(),
		"bro"
	);
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
