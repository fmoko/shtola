# Ware

Immutable middleware chains.

Ware allows you to create middleware chains that pass through and modify a value
as they go along. You can imagine a middleware chain as something like this:

``` rust
let initial_value = 1;

fn middleware_1(value: i32) -> i32 {
	value + 1
}

fn middleware_2(value: i32) -> i32 {
	value * 5
}

let result = middleware_2(middleware_1(initial_value));
assert_eq!(result, 10);
```

Ware abstracts over this concept like such:

``` rust
use ware::Ware;

let mut middleware_chain: Ware<i32> = Ware::new();

middleware_chain.wrap(Box::new(|num| num + 1));
middleware_chain.wrap(Box::new(|num| num * 5));

let result = middleware_chain.run(1);
assert_eq!(result, 10);
```

Ware provides a single-argument struct (e.g. `Ware<i32>`) and a dual-argument
struct (e.g. `Ware2<i32, String>`).

Functions that get registered as middleware cannot directly modify their
variables, as they have to by of the `Fn` trait. I would
recommend using immutable data structures that are efficient when duplicating values.

## Documentation

The documentation is available at https://docs.rs/ware.

## License

Ware is licensed under the Prosperity Public License (see [LICENSE](./LICENSE)).
You cannot use it commercially without my permission.
