use std::fmt::{Display, Write};

#[derive(Clone, Debug)]
pub struct Location {
	line_z: usize,
	col_z: usize,
	file: String,
}

impl Location {
	pub const fn new_z(line_z: usize, col_z: usize, file: String) -> Self {
		Self {
			line_z,
			col_z,
			file,
		}
	}

	pub fn new(line: usize, col: usize, file: String) -> Self {
		assert_ne!(line, 0);
		assert_ne!(col, 0);

		Self {
			line_z: line - 1,
			col_z: col - 1,
			file,
		}
	}

	pub fn line(&self) -> usize {
		self.line_z + 1
	}

	pub fn column(&self) -> usize {
		self.col_z + 1
	}

	pub fn file(&self) -> &str {
		&self.file
	}
}

impl Display for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.file())?;
		f.write_char(':')?;
		self.line().fmt(f)?;
		f.write_char(':')?;
		self.column().fmt(f)
	}
}
