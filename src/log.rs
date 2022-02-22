use crate::utils::config::TRIM_OUTPUT;
use std::{
	io::{stdout, Write},
	sync::atomic::Ordering,
};
use term_size;
use yansi::{Color, Style};

pub const LABEL_WIDTH: usize = 12;

/// Print a message with a label, add a carriage return at the end and flush the stdout
pub fn print_label<S: Into<String>>(label: OutputLabel, message: S) {
	print!("{}\r", pretty_output(label, message));

	stdout().flush().unwrap_or_else(|_| {
		println_label(OutputLabel::Error, "Could not flush stdout");
	});
}

/// Print a message with no label
pub fn println<S: Into<String>>(message: S) {
	println_label(OutputLabel::None, message);
}

/// Print a message with the specified label
pub fn println_label<S: Into<String>>(label: OutputLabel, message: S) {
	match label {
		OutputLabel::Error => {
			eprintln!("{}", pretty_output(label, message));
		}
		_ => {
			println!("{}", pretty_output(label, message));
		}
	}
}

/// The enum of possible output labels
#[allow(dead_code)]
pub enum OutputLabel<'a> {
	Error,
	Warning,
	Info(&'a str),
	Success(&'a str),
	Custom(&'a str, Color),
	Prompt(&'a str),
	None,
}

/// Pretty a message with a given label and a given message colour
pub fn pretty_output<S: Into<String>>(label: OutputLabel, message: S) -> String {
	let (label, label_color) = match label {
		OutputLabel::Error => (String::from("Error"), Color::Red),
		OutputLabel::Warning => (String::from("Warn"), Color::Yellow),
		OutputLabel::Info(info) => (String::from(info), Color::Blue),
		OutputLabel::Success(success) => (String::from(success), Color::Green),
		OutputLabel::Custom(custom, custom_colour) => (String::from(custom), custom_colour),
		OutputLabel::Prompt(prompt) => (String::from(prompt), Color::Yellow),
		OutputLabel::None => (String::from(""), Color::White),
	};

	let term_width = get_term_width();
	let message = message.into();
	let message_len = &message.len();

	match TRIM_OUTPUT.load(Ordering::Acquire) {
		true => format!(
			"{}{} {}{}",
			" ".repeat(LABEL_WIDTH - label.len()),
			Style::new(label_color).bold().paint(label),
			shorten(message, term_width - LABEL_WIDTH - 1),
			" ".repeat(term_width - LABEL_WIDTH - message_len - 1),
		),
		false => format!("{} {}", label, message),
	}
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

	// Break the message at half of the available width
	// Better for readability than at the end
	let break_index = max_width / 2;

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
