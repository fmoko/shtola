pub struct Ware<R> {
	pub fns: Vec<Box<dyn Fn(R) -> R>>,
}

impl<R> Ware<R> {
	pub fn new() -> Ware<R> {
		let vec: Vec<Box<dyn Fn(R) -> R>> = Vec::new();
		Ware { fns: vec }
	}

	pub fn wrap(&mut self, func: Box<dyn Fn(R) -> R>) {
		self.fns.push(func);
	}

	pub fn run(&self, arg: R) -> R {
		self.fns.iter().fold(arg, |acc, func| func(acc))
	}
}

pub struct Ware2<R, S> {
	pub fns: Vec<Box<dyn Fn(R, S) -> (R, S)>>,
}

impl<R, S> Ware2<R, S> {
	pub fn new() -> Ware2<R, S> {
		let vec: Vec<Box<dyn Fn(R, S) -> (R, S)>> = Vec::new();
		Ware2 { fns: vec }
	}

	pub fn wrap(&mut self, func: Box<dyn Fn(R, S) -> (R, S)>) {
		self.fns.push(func);
	}

	pub fn run (&self, arg1: R, arg2: S) -> (R, S) {
		self.fns.iter().fold((arg1, arg2), |acc, func| func(acc.0, acc.1))
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
