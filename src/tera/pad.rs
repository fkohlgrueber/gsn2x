use std::collections::HashMap;

use tera::{Error, Filter, Result, Value};

pub struct Pad;

impl Filter for Pad {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        pad(
            value
                .as_str()
                .ok_or_else(|| Error::msg("Only strings can be padded"))?,
            args.get("width")
                .ok_or_else(|| Error::msg("Parameter width missing"))?
                .as_u64()
                .ok_or_else(|| Error::msg("Parameter width is not an integer"))?,
        )
    }
}

///
/// Pad `s` with `width`.
///
fn pad(s: &str, width: u64) -> Result<Value> {
    Ok(Value::from(format!(
        "{:width$}{}",
        " ",
        s,
        width = width as usize
    )))
}

#[cfg(test)]
mod test {
    use super::*;
    use tera::ErrorKind;
    #[test]
    fn wrong_value_type() {
        let ww = Pad {};
        assert!(matches!(
            ww.filter(&Value::Null, &HashMap::<String, Value>::new()).err().unwrap().kind, ErrorKind::Msg(t) if t == "Only strings can be padded"
        ));
    }

    #[test]
    fn no_width() {
        let ww = Pad {};
        let map = HashMap::<String, Value>::new();
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width missing"
            )
        );
    }

    #[test]
    fn invalid_width() {
        let ww = Pad {};
        let mut map = HashMap::<String, Value>::new();
        map.insert("width".to_owned(), Value::String("xyz".to_owned()));
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width is not an integer")
        );
    }
}