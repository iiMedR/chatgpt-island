use super::types::ValidationError;
pub const MAX_CHARACTERS: usize = 10_000;
pub fn validate(text: &str) -> Result<String, ValidationError> {
    if text.is_empty() {
        return Err(ValidationError::Empty);
    }
    if text.contains('\0') {
        return Err(ValidationError::ContainsNul);
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Whitespace);
    }
    if trimmed.chars().count() > MAX_CHARACTERS {
        return Err(ValidationError::TooLong);
    }
    Ok(trimmed.into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_languages_unicode_and_internal_line_breaks() {
        for text in [
            "English sample",
            "Français : déjà été",
            "مرحبا بالعالم",
            "one\r\ntwo\nthree",
            "é🙂𐍈",
        ] {
            assert_eq!(validate(&format!("  {text}\n")).unwrap(), text);
        }
    }
    #[test]
    fn rejects_empty_whitespace_nul_and_oversize_without_truncation() {
        assert_eq!(validate(""), Err(ValidationError::Empty));
        assert_eq!(validate(" \n\t\u{2003}"), Err(ValidationError::Whitespace));
        assert_eq!(validate("a\0b"), Err(ValidationError::ContainsNul));
        assert!(validate(&"🙂".repeat(MAX_CHARACTERS)).is_ok());
        assert_eq!(
            validate(&"🙂".repeat(MAX_CHARACTERS + 1)),
            Err(ValidationError::TooLong)
        );
    }
}
