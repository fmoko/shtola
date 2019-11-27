//! With Shtola, you can build your own static site generators easily. All that
//! Shtola itself does is read files and frontmatter, run them through a bunch
//! of user-provided plugins, and write the result back to disk.
//!
//! As a demonstration of Shtola's basic piping feature, see this example:
//! ```
//! use shtola::Shtola;
//!
//! let mut m = Shtola::new();
//! m.source("../fixtures/simple");
//! m.destination("../fixtures/dest");
//! m.clean(true);
//! m.build().unwrap();
//! ```
//!
//! A "plugin" is just a boxed function that takes an `IR` (intermediate
//! representation) struct and also returns an `IR` struct. The plugin may
//! modify the IR by using immutable modification:
//!
//! ```
//! use shtola::{Plugin, IR, ShFile};
//!
//! fn plugin() -> Plugin {
//!   Box::new(|ir: IR| {
//!     IR { files: ir.files.update("myFile".into(), ShFile::empty()), ..ir }
//!   })
//! }
//! ```

use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, info, trace};
use pathdiff::diff_paths;
use serde_json::json;
use std::default::Default;
use std::fs;
use std::time::Instant;
use std::io::{Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

pub use im::HashMap;
pub use log;
pub use serde_json as json;
pub use ware::Ware;

mod frontmatter;
#[cfg(test)]
mod tests;

/// The main library struct.
pub struct Shtola {
	ware: Ware<IR>,
	ir: IR,
}

impl Shtola {
	/// Creates a new empty Shtola struct.
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

	/// Appends glob-matched paths to the ignore list. If a glob path matches, the
	/// file is excluded from the IR.
	/// ```
	/// use shtola::Shtola;
	///
	/// let mut m = Shtola::new();
	/// m.ignores(&mut vec!["node_modules".into(), "vendor/bundle/".into()])
	/// ```
	pub fn ignores(&mut self, vec: &mut Vec<String>) {
		self.ir.config.ignores.append(vec);
		self.ir.config.ignores.dedup();
	}

	/// Sets the source directory to read from. Should be relative.
	pub fn source<T: Into<PathBuf>>(&mut self, path: T) {
		self.ir.config.source = fs::canonicalize(path.into()).unwrap();
	}

	/// Sets the destination path to write to. This directory will be created on
	/// calling this function if it doesn't exist.
	pub fn destination<T: Into<PathBuf> + Clone>(&mut self, path: T) {
		fs::create_dir_all(path.clone().into()).expect("Unable to create destination directory!");
		self.ir.config.destination = fs::canonicalize(path.into()).unwrap();
	}

	/// Sets whether the destination directory should be removed before building.
	/// The removal only happens once calling [`Shtola::build`](#method.build).
	/// Default is `false`.
	pub fn clean(&mut self, b: bool) {
		self.ir.config.clean = b;
	}

	/// Sets whether frontmatter should be parsed. Default is `true`.
	pub fn frontmatter(&mut self, b: bool) {
		self.ir.config.frontmatter = b;
	}

	/// Registers a new plugin function in its middleware chain.
	///
	/// ```
	/// use shtola::{Shtola, IR};
	///
	/// let mut m = Shtola::new();
	/// let plugin = Box::new(|ir: IR| ir);
	/// m.register(plugin);
	/// ```
	pub fn register(&mut self, func: Box<dyn Fn(IR) -> IR>) {
		self.ware.wrap(func);
	}

	/// Performs the build process. This does a couple of things:
	/// - If [`Shtola::clean`](#method.clean) is set, removes and recreates the
	///   destination directory
	/// - Reads from the source file and ignores files as it's been configured
	/// - Parses front matter for the remaining files
	/// - Runs the middleware chain, executing all plugins
	/// - Writes the result back to the destination directory
	pub fn build(&mut self) -> Result<IR, std::io::Error> {
		let now = Instant::now();
		info!("Starting Shtola");
		trace!("Starting IR config: {:?}", self.ir.config);
		if self.ir.config.clean {
			info!("Cleaning before build...");
			debug!("Removing {:?}", &self.ir.config.destination);
			fs::remove_dir_all(&self.ir.config.destination)?;
			debug!("Recreating {:?}", &self.ir.config.destination);
			fs::create_dir_all(&self.ir.config.destination)
				.expect("Unable to recreate destination directory!");
		}

		let mut builder = GlobSetBuilder::new();
		for item in &self.ir.config.ignores {
			builder.add(Glob::new(item).unwrap());
		}
		trace!("Globs: {:?}", &builder);
		let set = builder.build().unwrap();
		trace!("Globset: {:?}", &set);
		info!("Reading files...");
		let files = read_dir(&self.ir.config.source, self.ir.config.frontmatter, set)?;
		trace!("Files: {:?}", &files);

		self.ir.files = files;
		info!("Running plugins...");
		let result_ir = self.ware.run(self.ir.clone());
		trace!("Result IR: {:?}", &result_ir);
		write_dir(result_ir.clone(), &self.ir.config.destination)?;
		info!("Build done in {}s", now.elapsed().as_secs());
		Ok(result_ir)
	}
}

/// Convenience type to return from plugin functions.
pub type Plugin = Box<dyn Fn(IR) -> IR>;

/// The intermediate representation that's passed to plugins. Includes global
/// metadata, the files with frontmatter and the global config.
#[derive(Debug, Clone)]
pub struct IR {
	/// The filestate, contained in an `im::HashMap`.
	pub files: HashMap<PathBuf, ShFile>,
	/// The configuration.
	pub config: Config,
	/// Global metadata managed as a `HashMap` that keep JSON values as values.
	pub metadata: HashMap<String, json::Value>,
}

/// Configuration struct.
#[derive(Debug, Clone)]
pub struct Config {
	/// Files that are to be ignored.
	pub ignores: Vec<String>,
	/// Source to read from.
	pub source: PathBuf,
	/// Destination to write to.
	pub destination: PathBuf,
	/// Whether to clean the destination directory.
	pub clean: bool,
	/// Whether to parse frontmatter.
	pub frontmatter: bool,
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

/// Shtola's file representation, with frontmatter included.
#[derive(Debug, Clone)]
pub struct ShFile {
	/// The frontmatter.
	pub frontmatter: json::Value,
	/// The file contents (without frontmatter).
	pub content: Vec<u8>,
}

impl ShFile {
	/// Creates an empty ShFile. Useful for deleting files using
	/// [`HashMap::difference`](struct.HashMap.html#method.difference):
	///
	/// ```
	/// use shtola::{Plugin, IR, ShFile, HashMap};
	/// use std::path::PathBuf;
	///
	/// fn plugin() -> Plugin {
	///   Box::new(|ir: IR| {
	///     let mut deletion_hash: HashMap<PathBuf, ShFile> = HashMap::new();
	///     deletion_hash.insert("deleted-file.md".into(), ShFile::empty());
	///     IR { files: deletion_hash.difference(ir.files), ..ir }
	///   })
	/// }
	/// ```
	pub fn empty() -> ShFile {
		ShFile {
			frontmatter: json!(null),
			content: Vec::new(),
		}
	}
}

fn read_dir(
	source: &PathBuf,
	frontmatter: bool,
	set: GlobSet,
) -> Result<HashMap<PathBuf, ShFile>, std::io::Error> {
	let mut result = HashMap::new();
	let iters = WalkDir::new(source)
		.into_iter()
		.filter_entry(|e| {
			let path = diff_paths(e.path(), source).unwrap();
			trace!("Read Filter: {:?} matches? {}", &path, set.is_match(&path));
			!set.is_match(path)
		})
		.filter(|e| !e.as_ref().ok().unwrap().file_type().is_dir());
	for entry in iters {
		let entry = entry?;
		let path = entry.path();
		let file: ShFile;
		let mut content = String::new();
		debug!("Reading file at {:?}", &path);
		fs::File::open(path)?.read_to_string(&mut content)?;
		if frontmatter {
			let (matter, content) = frontmatter::lexer(&content);
			if matter.len() > 0 {
				debug!("Lexing frontmatter for {:?}", &path);
				trace!("Frontmatter: {:?}", &matter);
			}
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
		let dest_path = dest.join(&path);
		debug!("Writing {:?} to {:?}", &path, &dest_path);
		fs::create_dir_all(dest_path.parent().unwrap())
			.expect("Unable to create destination subdirectory!");
		fs::File::create(dest_path)?.write_all(&file.content)?;
	}
	Ok(())
}
