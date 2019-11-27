use shtola::Shtola;
use shtola_markdown::plugin as markdown;

fn main() {
	pretty_env_logger::init();
	let mut s = Shtola::new();
	s.source("fixtures/markdown");
	s.destination("fixtures/markdown/dest");
	s.clean(true);
	s.register(markdown());
	s.build().unwrap();
}
