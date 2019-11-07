use yaml_rust::{Yaml, YamlLoader};

#[allow(dead_code)]
pub fn lexer(text: &str) -> (String, String) {
	if text.starts_with("---\n") {
		let slice_after_marker = &text[4..];
		let marker_end = slice_after_marker.find("---\n").unwrap();
		let yaml_slice = &text[4..marker_end + 4];
		let content_slice = &text[marker_end + 2 * 4..];
		(
			yaml_slice.trim().to_string(),
			content_slice.trim().to_string(),
		)
	} else {
		(String::new(), text.to_string())
	}
}

#[allow(dead_code)]
pub fn to_yaml(matter: &str) -> Vec<Yaml> {
	if matter.len() == 0 {
		return Vec::new();
	}
	YamlLoader::load_from_str(matter).unwrap()
}
