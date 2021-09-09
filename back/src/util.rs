pub fn expand_underscores(src: &str, length: usize) -> String {
	assert!(length > 0);

	if length == 1 {
		return src.to_owned();
	}

	src.chars()
		.map(|c| {
			if c == '_' {
				"_".repeat(length)
			} else {
				c.to_string()
			}
		})
		.collect::<Vec<_>>()
		.concat()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[should_panic]
	fn test_expand_underscores_panics_if_length_0() {
		let src = "Hello _, I'm _ years old";
		expand_underscores(src, 0);
	}

	#[test]
	fn test_expand_underscores() {
		let src = "Hello _, I'm _ years old";
		assert_eq!(&expand_underscores(src, 1), src);
		assert_eq!(&expand_underscores(src, 3), "Hello ___, I'm ___ years old");
	}

	#[test]
	fn test_expand_underscores_no_underscores() {
		let src = "Hello world!";
		assert_eq!(&expand_underscores(src, 3), "Hello world!");
	}
}
