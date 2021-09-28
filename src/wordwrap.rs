use std::collections::HashMap;

use tera::{Error, Filter, Result, Value};

pub struct WordWrap;

impl Filter for WordWrap {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        wordwrap(
            value
                .as_str()
                .ok_or_else(|| Error::msg("Value is not a string"))?,
            args.get("width")
                .ok_or_else(|| Error::msg("Parameter width missing"))?
                .as_u64()
                .ok_or_else(|| Error::msg("Parameter width is not an integer"))?,
            args.get("wrapstr")
                .ok_or_else(|| Error::msg("Parameter wrapstr missing"))?
                .as_str()
                .ok_or_else(|| Error::msg("Parameter wrapstr is not a string"))?,
        )
    }
}

fn wordwrap(s: &str, width: u64, wrapstr: &str) -> Result<Value> {
    let mut out = Vec::<String>::new();
    for line in s.lines() {
        let mut cur_line = String::new();
        for word in line.split_ascii_whitespace() {
            if cur_line.len() + word.len() > width as usize {
                out.push(cur_line);
                cur_line = String::new();
            } else if !cur_line.is_empty() {
                cur_line.push(' ');
            }
            cur_line.push_str(word);
        }
        if !cur_line.is_empty() {
            out.push(cur_line);
        }
    }
    Ok(Value::from(out.join(wrapstr)))
}

#[cfg(test)]
mod test {
    // Since tera Error and ErrorKind do not implement PartialEq this workaround helped:
    // https://stackoverflow.com/questions/57234140/how-to-assert-io-errors-in-rust

    use tera::ErrorKind;

    use super::*;

    #[test]
    fn simple() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";
        let expected = Value::from(concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing\n",
            "elitr, sed diam nonumy eirmod tempor invidunt ut\n",
            "labore et dolore magna aliquyam erat, sed diam\n",
            "voluptua. At vero eos et accusam et justo duo\n",
            "dolores et ea rebum. Stet clita kasd gubergren, no\n",
            "sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet. Lorem ipsum dolor sit amet, consetetur\n",
            "sadipscing elitr, sed diam nonumy eirmod tempor\n",
            "invidunt ut labore et dolore magna aliquyam erat,\n",
            "sed diam voluptua. At vero eos et accusam et justo\n",
            "duo dolores et ea rebum. Stet clita kasd gubergren,\n",
            "no sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet."
        ));
        let out = wordwrap(input, 50, "\n").unwrap();
        assert_eq!(out, expected);
    }
    #[test]
    fn shorter() {
        let input = "Lorem ipsum dolor sit amet, consetetur";
        let expected = Value::from("Lorem ipsum dolor sit amet, consetetur".to_owned()); // make explicit heap allocation to prevent Short value
        let out = wordwrap(input, 50, "\n").unwrap();
        assert_eq!(out, expected);
    }
    #[test]
    fn wrapstring() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt";
        let expected = Value::from(concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing<br align=\"left\"/>",
            "elitr, sed diam nonumy eirmod tempor invidunt",
        ));
        let out = wordwrap(input, 50, "<br align=\"left\"/>").unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn withnewlines() {
        let input = "Lorem ipsum dolor sit amet,\nconsetetur sadipscing\nelitr, sed diam nonumy eirmod tempor invidunt";
        let expected = Value::from(concat!(
            "Lorem ipsum dolor sit amet,<br align=\"left\"/>",
            "consetetur sadipscing<br align=\"left\"/>",
            "elitr, sed diam nonumy eirmod tempor invidunt",
        ));
        let out = wordwrap(input, 50, "<br align=\"left\"/>").unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn no_value() {
        let ww = WordWrap {};
        assert!(matches!(
            ww.filter(&Value::Null, &HashMap::<String, Value>::new()).err().unwrap().kind, ErrorKind::Msg(t) if t == "Value is not a string"
        ));
    }

    #[test]
    fn no_width() {
        let ww = WordWrap {};
        let map = HashMap::<String, Value>::new();
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width missing"
            )
        );
    }

    #[test]
    fn invalid_width() {
        let ww = WordWrap {};
        let mut map = HashMap::<String, Value>::new();
        map.insert("width".to_owned(), Value::String("xyz".to_owned()));
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width is not an integer")
        );
    }

    #[test]
    fn no_wrapstr() {
        let ww = WordWrap {};
        let mut map = HashMap::<String, Value>::new();
        map.insert("width".to_owned(), Value::Number(tera::Number::from(42u64)));
        // wrapstr is missing
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter wrapstr missing")
        );
    }

    #[test]
    fn invalid_wrapstr() {
        let ww = WordWrap {};
        let mut map = HashMap::<String, Value>::new();
        map.insert("width".to_owned(), Value::Number(tera::Number::from(42u64)));
        map.insert("wrapstr".to_owned(), Value::Null);
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter wrapstr is not a string")
        );
    }
}
