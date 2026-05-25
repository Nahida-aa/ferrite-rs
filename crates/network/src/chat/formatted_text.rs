use std::ops::ControlFlow;

use super::style::Style;

/// A styled or unstyled text fragment.
///
/// `ControlFlow::Continue(())` means "keep iterating",
/// `ControlFlow::Break(())` means "stop iteration".
pub trait FormattedText {
    fn visit(&self, output: &mut dyn FnMut(&str) -> ControlFlow<()>) -> ControlFlow<()>;

    fn visit_styled(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> ControlFlow<()>,
        parent_style: &Style,
    ) -> ControlFlow<()>;
}

// --------------------------------------------------------------------------
// EMPTY sentinel
// --------------------------------------------------------------------------

struct EmptyText;

impl FormattedText for EmptyText {
    fn visit(&self, _output: &mut dyn FnMut(&str) -> ControlFlow<()>) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_styled(
        &self,
        _output: &mut dyn FnMut(&Style, &str) -> ControlFlow<()>,
        _parent_style: &Style,
    ) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

/// Singleton empty text.
pub const EMPTY: &dyn FormattedText = &EmptyText;

// --------------------------------------------------------------------------
// Literal (unstyled) text
// --------------------------------------------------------------------------

pub struct LiteralText {
    text: String,
}

impl LiteralText {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl FormattedText for LiteralText {
    fn visit(&self, output: &mut dyn FnMut(&str) -> ControlFlow<()>) -> ControlFlow<()> {
        output(&self.text)
    }

    fn visit_styled(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> ControlFlow<()>,
        parent_style: &Style,
    ) -> ControlFlow<()> {
        output(parent_style, &self.text)
    }
}

// --------------------------------------------------------------------------
// Styled text
// --------------------------------------------------------------------------

pub struct StyledText {
    text: String,
    style: Style,
}

impl StyledText {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self { text: text.into(), style }
    }
}

impl FormattedText for StyledText {
    fn visit(&self, output: &mut dyn FnMut(&str) -> ControlFlow<()>) -> ControlFlow<()> {
        output(&self.text)
    }

    fn visit_styled(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> ControlFlow<()>,
        parent_style: &Style,
    ) -> ControlFlow<()> {
        let merged = self.style.apply_to(parent_style);
        output(&merged, &self.text)
    }
}

// --------------------------------------------------------------------------
// Composite (multiple fragments)
// --------------------------------------------------------------------------

pub struct CompositeText {
    parts: Vec<Box<dyn FormattedText>>,
}

impl CompositeText {
    pub fn new(parts: Vec<Box<dyn FormattedText>>) -> Self {
        Self { parts }
    }
}

impl FormattedText for CompositeText {
    fn visit(&self, output: &mut dyn FnMut(&str) -> ControlFlow<()>) -> ControlFlow<()> {
        for part in &self.parts {
            part.visit(output)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_styled(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> ControlFlow<()>,
        parent_style: &Style,
    ) -> ControlFlow<()> {
        for part in &self.parts {
            part.visit_styled(output, parent_style)?;
        }
        ControlFlow::Continue(())
    }
}

// --------------------------------------------------------------------------
// Utility methods (akin to default methods on the Java interface)
// --------------------------------------------------------------------------

pub fn get_string(text: &dyn FormattedText) -> String {
    let mut buf = String::new();
    let _ = text.visit(&mut |chunk| {
        buf.push_str(chunk);
        ControlFlow::Continue(())
    });
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty_string() {
        assert_eq!(get_string(EMPTY), "");
    }

    #[test]
    fn literal_text() {
        let t = LiteralText::new("hello");
        assert_eq!(get_string(&t), "hello");
    }

    #[test]
    fn composite_text() {
        let parts: Vec<Box<dyn FormattedText>> = vec![
            Box::new(LiteralText::new("foo")),
            Box::new(LiteralText::new("bar")),
        ];
        let c = CompositeText::new(parts);
        assert_eq!(get_string(&c), "foobar");
    }

    #[test]
    fn styled_text_visit_unstyled() {
        let t = StyledText::new("styled", Style);
        let mut buf = String::new();
        let _ = t.visit(&mut |chunk| {
            buf.push_str(chunk);
            ControlFlow::Continue(())
        });
        assert_eq!(buf, "styled");
    }

    #[test]
    fn early_stop() {
        let parts: Vec<Box<dyn FormattedText>> = vec![
            Box::new(LiteralText::new("a")),
            Box::new(LiteralText::new("b")),
            Box::new(LiteralText::new("c")),
        ];
        let c = CompositeText::new(parts);
        let mut buf = String::new();
        let _ = c.visit(&mut |chunk| {
            buf.push_str(chunk);
            if chunk == "b" {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
        assert_eq!(buf, "ab");
    }
}
