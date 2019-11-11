use serde_json::{json, Value};
use serde_yaml::from_str;

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

pub fn to_json(matter: &str) -> Value {
	if matter.len() == 0 {
		return json!(null);
	}
	let yaml: Value = from_str(matter).unwrap();
	yaml
}
