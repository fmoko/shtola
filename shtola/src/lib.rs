use ware::Ware;
use im::HashMap;
use walkdir::WalkDir;
use yaml_rust::Yaml;
use pathdiff::diff_paths;
use std::path::PathBuf;
use std::fs::{File, canonicalize};
use std::io::Read;
use std::default::Default;

mod frontmatter;

pub struct Shtola {
	ware: Ware<IR>,
	ir: IR,
}

impl Shtola {
	pub fn new() -> Shtola {
		let config: Config = Default::default();
		let ir = IR { files: HashMap::new(), config };
		Shtola { ware: Ware::new(), ir }
	}

	pub fn ignores(&mut self, vec: &mut Vec<PathBuf>) {
		self.ir.config.ignores.append(vec);
		self.ir.config.ignores.dedup();
	}

	pub fn source<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.source = canonicalize(path.into()).unwrap();
	}

	pub fn destination<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.destination = canonicalize(path.into()).unwrap();
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
		// clean if set
		let files = read_dir(&self.ir.config.source)?;
		self.ir.files = files;
		let result_ir = self.ware.run(self.ir.clone());
		// write files
		Ok(result_ir)
	}
}

#[derive(Debug, Clone)]
pub struct IR {
	files: HashMap<PathBuf, ShFile>,
	config: Config,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
	ignores: Vec<PathBuf>,
	source: PathBuf,
	destination: PathBuf,
	clean: bool,
	frontmatter: bool,
}


#[derive(Debug, Clone)]
pub struct ShFile {
	frontmatter: Vec<Yaml>,
	content: Vec<u8>,
}

fn read_dir(source: &PathBuf) -> Result<HashMap<PathBuf, ShFile>, std::io::Error> {
	let mut result = HashMap::new();
	let iters = WalkDir::new(source)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|e| !e.path().is_dir());
	for entry in iters {
		let path = entry.path();
		let mut content = String::new();
		File::open(path)?.read_to_string(&mut content)?;
		let (matter, content) = frontmatter::lexer(&content);
		let yaml = frontmatter::to_yaml(&matter);
		let file = ShFile {
			frontmatter: yaml,
			content: content.into(),
		};
		let rel_path = diff_paths(path, source).unwrap();
		result.insert(rel_path, file);
	}
	Ok(result)
}

#[test]
fn read_works() {
	let mut s = Shtola::new();
	s.source("../fixtures/simple");
	s.destination("./");
	s.clean(true);
	let r = s.build().unwrap();
	assert_eq!(r.files.len(), 1);
	let keys: Vec<&PathBuf> = r.files.keys().collect();
	assert_eq!(keys[0].to_str().unwrap(), "hello.txt");
}
