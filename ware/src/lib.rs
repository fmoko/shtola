//! Ware provides immutable middleware abstractions. Basically, it means that
//! you can pass one variable through a series of functions that all have the
//! ability to modify this variable, therefore sending this modified version of
//! it further down the chain.
//!
//! Ware is used like this:
//!
//! ```
//! use ware::Ware;
//!
//! fn main() {
//!     let mut chain: Ware<i32> = Ware::new();
//!     chain.wrap(Box::new(|num| num * 10));
//!     chain.wrap(Box::new(|num| num - 2));
//!     let result = chain.run(5);
//!     assert_eq!(result, 48);
//! }
//! ```
//!
//! Ware also provides a version of itself that can pass through two variables
//! at once, since Rust doesn't support variadic functions (functions that can
//! have multiple numbers of arguments). In middleware functions for these,
//! you return a 2-tuple instead of a single value:
//!
//! ```
//! use ware::Ware2;
//!
//! fn main() {
//!     let mut chain: Ware2<i32, i32> = Ware2::new();
//!     chain.wrap(Box::new(|num1, num2| (num1 - 4, num2 + 3)));
//!     let (res1, res2) = chain.run(10, 10);
//!     assert_eq!(res1, 6);
//!     assert_eq!(res2, 13);
//! }
//! ```

/// A middleware chain that can pass through one argument.
pub struct Ware<R> {
	/// The internal list of middleware functions.
	pub fns: Vec<Box<dyn Fn(R) -> R>>,
}

impl<R> Ware<R> {
	/// Create a new middleware chain with a given type.
	///
	/// # Example
	/// ```
	/// use ware::Ware;
	/// let mut chain: Ware<String> = Ware::new();
	/// ```
	pub fn new() -> Ware<R> {
		let vec: Vec<Box<dyn Fn(R) -> R>> = Vec::new();
		Ware { fns: vec }
	}

	/// Add a new middleware function to the internal function list. This function
	/// must be of the `Fn` trait, take the specified type and return the same
	/// specified type. It also has to be boxed for memory safety reasons.
	///
	/// # Example
	/// ```
	/// use ware::Ware;
	/// let mut chain: Ware<String> = Ware::new();
	/// chain.wrap(Box::new(|st| {
	///     let mut s = st.clone();
	///     s.push('a');
	///     s
	/// }))
	/// ```
	pub fn wrap(&mut self, func: Box<dyn Fn(R) -> R>) {
		self.fns.push(func);
	}

	/// Run the registered middleware functions with the given value to pass
	/// through. Returns whatever the last registered middleware function
	/// returns.
	pub fn run(&self, arg: R) -> R {
		self.fns.iter().fold(arg, |acc, func| func(acc))
	}
}

/// A middleware chain that can pass through two arguments.
pub struct Ware2<R, S> {
	/// The internal list of middleware functions.
	pub fns: Vec<Box<dyn Fn(R, S) -> (R, S)>>,
}

impl<R, S> Ware2<R, S> {
	/// Create a new middleware chain with the two given types.
	///
	/// # Example
	/// ```
	/// use ware::Ware2;
	/// let mut chain: Ware2<String, i32> = Ware2::new();
	/// ```
	pub fn new() -> Ware2<R, S> {
		let vec: Vec<Box<dyn Fn(R, S) -> (R, S)>> = Vec::new();
		Ware2 { fns: vec }
	}

	/// Add a new middleware function to the internal function list. This function
	/// must be of the `Fn` trait, take the specified types in order and return
	/// a tuple of the same specified types. It also has to be boxed for memory
	/// safety reasons.
	///
	/// # Example
	/// ```
	/// use ware::Ware2;
	/// let mut chain: Ware2<String, i32> = Ware2::new();
	/// chain.wrap(Box::new(|st, num| {
	///     let mut s = st.clone();
	///     s.push('a');
	///     (s, num + 1)
	/// }))
	/// ```
	pub fn wrap(&mut self, func: Box<dyn Fn(R, S) -> (R, S)>) {
		self.fns.push(func);
	}

	/// Run the registered middleware functions with the given value to pass
	/// through. Returns whatever the last registered middleware function
	/// returns.
	pub fn run(&self, arg1: R, arg2: S) -> (R, S) {
		self.fns
			.iter()
			.fold((arg1, arg2), |acc, func| func(acc.0, acc.1))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let value = 1;
		let mut w: Ware<i32> = Ware::new();
		w.wrap(Box::new(|num| num + 1));
		assert_eq!(w.run(value), 2);
	}

	#[test]
	fn it_is_immutable() {
		let value = 1;
		let closure = |num| {
			let num = num + 1;
			num
		};
		let mut w: Ware<i32> = Ware::new();
		w.wrap(Box::new(closure));
		assert_eq!(w.run(value), 2);
		assert_eq!(value, 1);
	}

	#[test]
	fn ware2_works() {
		let val1 = 2;
		let val2 = "a".to_string();
		let closure = |num: i32, st: String| {
			let num = num - 1;
			let mut string = String::new();
			string.push_str(&st);
			string.push('b');
			(num, string)
		};
		let mut w: Ware2<i32, String> = Ware2::new();
		w.wrap(Box::new(closure));
		assert_eq!(w.run(val1, val2), (1, String::from("ab")));
	}
}
