use quick_xml::events::Event;
use quick_xml::Reader;

use super::XmlError;

/// Result of reading an element's content: either text or nested children.
enum TextOrChildren {
    Text(String),
    Children(Vec<(String, String)>),
}

/// A wrapper around `quick_xml::Reader` that provides OFX-specific reading helpers.
pub struct OfxReader<'a> {
    reader: Reader<&'a [u8]>,
    buf: Vec<u8>,
}

impl<'a> OfxReader<'a> {
    /// Create a new reader from an XML string.
    #[must_use]
    pub fn new(xml: &'a str) -> Self {
        let mut reader = Reader::from_reader(xml.as_bytes());
        reader.config_mut().trim_text(true);
        Self {
            reader,
            buf: Vec::with_capacity(256),
        }
    }

    /// Read the next XML event, skipping comments and declarations.
    ///
    /// # Errors
    ///
    /// Returns `XmlError::MalformedXml` if the underlying XML is invalid.
    pub fn next_event(&mut self) -> Result<Event<'_>, XmlError> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Comment(_) | Event::Decl(_) | Event::PI(_)) => {},
                Ok(event) => return Ok(event.into_owned()),
                Err(e) => {
                    return Err(XmlError::MalformedXml {
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    /// Read the text content of the current element.
    /// Expects the reader to be positioned just after a Start event.
    /// Consumes up to and including the matching End event.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML is malformed or an unexpected element is encountered.
    pub fn read_text(&mut self, tag_name: &str) -> Result<String, XmlError> {
        let mut text = String::new();
        loop {
            match self.next_event()? {
                Event::Text(t) => {
                    text.push_str(&String::from_utf8_lossy(&t));
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == tag_name {
                        return Ok(text);
                    }
                    return Err(XmlError::UnexpectedElement {
                        expected: tag_name.to_owned(),
                        found: format!("/{name}"),
                    });
                }
                Event::Eof => {
                    return Err(XmlError::MalformedXml {
                        message: format!("unexpected EOF while reading <{tag_name}>"),
                    });
                }
                _ => {
                    // Nested elements within a text field are unexpected
                    // but we try to be lenient -- skip them
                }
            }
        }
    }

    /// Skip remaining children and consume the end tag for the given parent.
    /// Use when you've already found what you need inside an aggregate and want
    /// to advance past the closing tag.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML is malformed or EOF is reached unexpectedly.
    pub fn skip_to_end(&mut self, parent_tag: &str) -> Result<(), XmlError> {
        let mut depth: u32 = 1;
        loop {
            match self.next_event()? {
                Event::Start(_) => depth += 1,
                Event::End(e) => {
                    depth -= 1;
                    if depth == 0 {
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        if name == parent_tag {
                            return Ok(());
                        }
                    }
                }
                Event::Eof => {
                    return Err(XmlError::MalformedXml {
                        message: format!("unexpected EOF while skipping to end of <{parent_tag}>"),
                    });
                }
                _ => {}
            }
        }
    }

    /// Skip the current element and all its children.
    /// The reader should be positioned just after a Start event.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML is malformed or EOF is reached unexpectedly.
    pub fn skip_element(&mut self, tag_name: &str) -> Result<(), XmlError> {
        let mut depth: u32 = 1;
        loop {
            match self.next_event()? {
                Event::Start(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == tag_name {
                        depth += 1;
                    }
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == tag_name {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(());
                        }
                    }
                }
                Event::Eof => {
                    return Err(XmlError::MalformedXml {
                        message: format!("unexpected EOF while skipping <{tag_name}>"),
                    });
                }
                _ => {}
            }
        }
    }

    /// Read and collect all child elements of the current aggregate, flattening
    /// nested aggregates into the result.
    ///
    /// Returns a list of (`tag_name`, `text_content`) pairs. For nested aggregates,
    /// the children are recursively flattened into the list (the aggregate
    /// wrapper tag itself is not included).
    ///
    /// Stops when the closing tag matching `parent_tag` is found.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML is malformed or EOF is reached unexpectedly.
    pub fn read_child_elements(
        &mut self,
        parent_tag: &str,
    ) -> Result<Vec<(String, String)>, XmlError> {
        let mut elements = Vec::new();
        self.read_children_recursive(parent_tag, &mut elements)?;
        Ok(elements)
    }

    fn read_children_recursive(
        &mut self,
        parent_tag: &str,
        elements: &mut Vec<(String, String)>,
    ) -> Result<(), XmlError> {
        loop {
            match self.next_event()? {
                Event::Start(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    // Peek ahead: try to read as text first
                    match self.read_text_or_children(&name)? {
                        TextOrChildren::Text(text) => {
                            elements.push((name, text));
                        }
                        TextOrChildren::Children(children) => {
                            // Flatten the aggregate's children into our list
                            elements.extend(children);
                        }
                    }
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == parent_tag {
                        return Ok(());
                    }
                }
                Event::Eof => {
                    return Err(XmlError::MalformedXml {
                        message: format!("unexpected EOF while reading children of <{parent_tag}>"),
                    });
                }
                _ => {}
            }
        }
    }

    /// After reading a Start event, determine if the element contains text or
    /// child elements. Returns either the text content or a flattened list of
    /// child (tag, text) pairs.
    fn read_text_or_children(
        &mut self,
        tag_name: &str,
    ) -> Result<TextOrChildren, XmlError> {
        let mut text = String::new();
        loop {
            match self.next_event()? {
                Event::Text(t) => {
                    text.push_str(&String::from_utf8_lossy(&t));
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == tag_name {
                        return Ok(TextOrChildren::Text(text));
                    }
                    // Unexpected end tag
                    return Err(XmlError::UnexpectedElement {
                        expected: tag_name.to_owned(),
                        found: format!("/{name}"),
                    });
                }
                Event::Start(e) => {
                    // This element has children -- it's an aggregate.
                    // Read this child and all subsequent children recursively.
                    let child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut children = Vec::new();

                    // Read the first child we just found
                    match self.read_text_or_children(&child_name)? {
                        TextOrChildren::Text(child_text) => {
                            children.push((child_name, child_text));
                        }
                        TextOrChildren::Children(grandchildren) => {
                            children.extend(grandchildren);
                        }
                    }

                    // Continue reading remaining children of this aggregate
                    self.read_children_recursive(tag_name, &mut children)?;
                    return Ok(TextOrChildren::Children(children));
                }
                Event::Eof => {
                    return Err(XmlError::MalformedXml {
                        message: format!("unexpected EOF while reading <{tag_name}>"),
                    });
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_simple_element() {
        let xml = "<CODE>0</CODE>";
        let mut reader = OfxReader::new(xml);
        // Read the start event
        match reader.next_event().unwrap() {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                assert_eq!(name, "CODE");
                let text = reader.read_text("CODE").unwrap();
                assert_eq!(text, "0");
            }
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn skip_element_works() {
        let xml = "<PARENT><CHILD>text</CHILD></PARENT><NEXT>after</NEXT>";
        let mut reader = OfxReader::new(xml);
        // Read PARENT start
        let _ = reader.next_event().unwrap();
        // Skip PARENT
        reader.skip_element("PARENT").unwrap();
        // Should now be at NEXT
        match reader.next_event().unwrap() {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                assert_eq!(name, "NEXT");
            }
            other => panic!("expected Start(NEXT), got {other:?}"),
        }
    }

    #[test]
    fn read_child_elements_basic() {
        let xml = "<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>";
        let mut reader = OfxReader::new(xml);
        // Consume the STATUS start tag
        let _ = reader.next_event().unwrap();
        let children = reader.read_child_elements("STATUS").unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], ("CODE".to_owned(), "0".to_owned()));
        assert_eq!(children[1], ("SEVERITY".to_owned(), "INFO".to_owned()));
    }
}
