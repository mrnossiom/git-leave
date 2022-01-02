use term_size;
use yansi::{Color, Style};

pub const LABEL_WIDTH: usize = 12;

pub fn println(message: &str) {
	println!("{}", pretty_output("", Color::White, message));
}

pub fn println_label<S: Into<String>>(label: S, label_colour: Color, message: S) {
	println!("{}", pretty_output(label, label_colour, message));
}

/// Pretty a message with a given label and a given message colour
pub fn pretty_output<S: Into<String>>(label: S, label_colour: Color, message: S) -> String {
	let term_width = get_term_width();
	let message = shorten(message.into(), term_width - LABEL_WIDTH - 1);
	let label = label.into();

	if label.len() > LABEL_WIDTH {
		panic!("Label {} too long", label);
	}

	return format!(
		"{}{} {}{}",
		" ".repeat(LABEL_WIDTH - label.len()),
		Style::new(label_colour).bold().paint(label),
		message,
		" ".repeat(term_width - LABEL_WIDTH - message.len() - 1),
	);
}

/// Shortens a message by omitting the middle part and replacing it with '...'
///
/// If the given message is shorter than the available width, the
/// original message will be returned
fn shorten(message: String, max_width: usize) -> String {
	let len = message.len();

	if len <= max_width {
		return message;
	}

	// Break the message at 3/4 of the available width
	let break_index = (max_width / 4) * 3;

	return [
		message.chars().take(break_index).collect(),
		"...".to_owned(),
		message
			.chars()
			.skip(len - max_width + break_index + 3)
			.collect(),
	]
	.join("");
}

fn get_term_width() -> usize {
	if let Some((width, _)) = term_size::dimensions() {
		width
	} else {
		80
	}
}
