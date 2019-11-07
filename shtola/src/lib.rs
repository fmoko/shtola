use ware::Ware;
use im::HashMap;
use std::path::PathBuf;
use std::default::Default;

pub struct Shtola {
	ware: Ware<IR>,
	ir: IR,
}

impl Shtola {
	pub fn new<T: Into<PathBuf>>(dir: T) -> Shtola {
		let mut config: Config = Default::default();
		config.directory = dir.into();
		let ir = IR { files: HashMap::new(), config };
		Shtola { ware: Ware::new(), ir }
	}

	pub fn ignores(&mut self, vec: &mut Vec<PathBuf>) {
		self.ir.config.ignores.append(vec);
		self.ir.config.ignores.dedup();
	}

	pub fn source<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.source = path.into();
	}

	pub fn destination<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.destination = path.into();
	}

	pub fn clean(&mut self, b: bool) {
		self.ir.config.clean = b;
	}

	pub fn frontmatter(&mut self, b: bool) {
		self.ir.config.frontmatter = b;
	}

	pub fn r#use(&mut self, func: Box<dyn Fn(IR) -> IR>) {
		self.ware.wrap(func);
	}

	pub fn build(&mut self) -> IR {
		// if clean is set, remove dest dir
		// read files
		let result_ir = self.ware.run(self.ir.clone());
		// write files
		// return ir
		result_ir
	}
}

#[derive(Clone)]
pub struct IR {
	files: HashMap<PathBuf, ShFile>,
	config: Config,
}

#[derive(Clone, Default)]
pub struct Config {
	ignores: Vec<PathBuf>,
	directory: PathBuf,
	source: PathBuf,
	destination: PathBuf,
	clean: bool,
	frontmatter: bool,
}


#[derive(Clone)]
pub struct ShFile {
	frontmatter: HashMap<String, String>,
	content: Vec<u8>,
}
