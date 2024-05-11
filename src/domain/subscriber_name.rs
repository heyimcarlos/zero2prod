use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    // returns a `SubscriberName` if validation passes.
    // Otherwise, panic.
    pub fn parse(value: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = value.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character `å` is a single grapheme, but it is composed of two characters
        // (a and ⚬)
        let is_too_long = value.graphemes(true).count() > 256;

        // Iterate over all characters of input `value` to check if any of them
        // matches one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|v| forbidden_characters.contains(&v));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            return Err(format!("{} is not a valid subscriber name", value));
        }
        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "å".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for c in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            assert_err!(SubscriberName::parse(c.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Pedro Almen".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
