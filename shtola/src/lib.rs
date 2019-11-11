use globset::{Glob, GlobSetBuilder};
use pathdiff::diff_paths;
use serde_json::json;
use std::default::Default;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

pub use im::HashMap;
pub use serde_json as json;
pub use ware::Ware;

mod frontmatter;
#[cfg(test)]
mod tests;

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
			metadata: HashMap::new(),
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
	metadata: HashMap<String, json::Value>,
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
	frontmatter: json::Value,
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
			let json = frontmatter::to_json(&matter);
			file = ShFile {
				frontmatter: json,
				content: content.into(),
			};
		} else {
			file = ShFile {
				frontmatter: json!(null),
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
