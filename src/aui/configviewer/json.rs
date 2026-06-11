//! # JSON output of a config
//!

use super::ConfigViewer;
use serde::ser::Serialize;
use serde_json::json;
use serde_json::value::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

/// Something that is standard HashMap but easy to convert to  JsonObject
type JsonLikeObj = HashMap<String, JsonValue>;

/// Display a config in JavaScript Object Notation (JSON) output
///
/// Typically on a web server
pub struct JsonConfigViewer<W: Write> {
    writer: W,
    stack: Vec<(String, Rc<RefCell<JsonLikeObj>>)>,
    pretty: bool,
}

impl<W: Write> JsonConfigViewer<W> {
    pub fn new(writer: W, pretty: bool) -> Self {
        let root = Rc::new(RefCell::new(HashMap::new()));
        Self {
            writer,
            pretty,
            stack: vec![(String::new(), root)],
        }
    }
}

impl<W: Write> ConfigViewer for JsonConfigViewer<W> {
    fn emit_kv(&mut self, key: &str, value: &JsonValue) {
        self.current()
            .borrow_mut()
            .insert(key.to_string(), value.clone());
    }

    fn emit_table(&mut self, key: &str, headers: &[&str], rows: &[Vec<JsonValue>]) {
        let mut json_values: Vec<HashMap<&str, &JsonValue>> = Vec::with_capacity(rows.len());

        for row in rows {
            let entry: HashMap<&str, &JsonValue> = row
                .iter()
                .enumerate()
                .take(headers.len())
                .map(|(j, cell)| (headers[j], cell))
                .collect();
            json_values.push(entry);
        }
        let as_json = json!(json_values);
        self.current().borrow_mut().insert(key.to_string(), as_json);
    }

    fn emit_section_start(&mut self, name: &str) {
        // Create an empty child node and push it onto the stack.
        // We don't insert it into the parent yet - we do that in
        // emit_section_end once it's fully populated.
        let child = Rc::new(RefCell::new(HashMap::new()));
        self.stack.push((String::from(name), child));
    }
    fn emit_section_end(&mut self) {
        // Pop the completed child node off the stack.
        let (name, child) = self.stack.pop().unwrap();

        // Convert the child's contents to a JsonValue::Object and
        // attach it to the (now-current) parent under its name.

        let child_map: JsonLikeObj = Rc::into_inner(child)
            .expect("child node still has other owners - this is a bug")
            .into_inner();

        let child_value = JsonValue::Object(child_map.into_iter().collect());

        self.current().borrow_mut().insert(name, child_value);
    }

    fn emit_comment(&mut self, _comment: &str) {}

    fn finish(&mut self) {
        // All open section shall be closed, thus stack should only have the root
        assert_eq!(self.stack.len(), 1);

        let json_obj = json!(self.root().as_ref());
        if self.pretty {
            let mut ser = serde_json::Serializer::pretty(&mut self.writer);

            json_obj.serialize(&mut ser).unwrap();

            let writer = ser.into_inner();

            writer.write_all("\n".as_bytes()).unwrap();
        } else {
            self.writer
                .write_all(json_obj.to_string().as_bytes())
                .unwrap();
        }
    }
}

impl<W: Write> JsonConfigViewer<W> {
    fn root(&self) -> Rc<RefCell<JsonLikeObj>> {
        // The stack is never empty (root is always present)
        self.stack.first().unwrap().1.clone()
    }

    fn current(&self) -> Rc<RefCell<JsonLikeObj>> {
        // The stack is never empty (root is always present)
        self.stack.last().unwrap().1.clone()
    }
}
