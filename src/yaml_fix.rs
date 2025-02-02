use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;

// Copied and adapted from https://serde.rs/deserialize-map.html
// to work around an issue in serde_yaml that does not check for duplicate keys in input YAML.
// Duplicate keys are no valid YAML but this is ignored by serde_yaml.

#[derive(Default, PartialEq)]
pub struct MyMap<K, V>(BTreeMap<K, V>)
where
    K: Ord;

impl<K: Ord + Debug, V: Debug> Debug for MyMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<K: Ord, V> MyMap<K, V> {
    pub fn new() -> MyMap<K, V> {
        MyMap(BTreeMap::<K, V>::new())
    }
}

impl<K, V> std::ops::Deref for MyMap<K, V>
where
    K: Ord,
{
    type Target = BTreeMap<K, V>;
    fn deref(&self) -> &BTreeMap<K, V> {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for MyMap<K, V>
where
    K: Ord,
{
    fn deref_mut(&mut self) -> &mut BTreeMap<K, V> {
        &mut self.0
    }
}

struct MyMapVisitor<K, V>
where
    K: Ord,
{
    marker: PhantomData<fn() -> MyMap<K, V>>,
}

impl<K: Ord, V> MyMapVisitor<K, V> {
    fn new() -> Self {
        MyMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for MyMapVisitor<K, V>
where
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    type Value = MyMap<K, V>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with unique keys")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = MyMap::new();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            let errmsg = format!("Element {} is already existing", key);
            if map.0.insert(key, value).is_some() {
                return Err(serde::de::Error::custom(errmsg));
            }
        }

        Ok(map)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, K, V> Deserialize<'de> for MyMap<K, V>
where
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(MyMapVisitor::new())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn format() {
        let btm = BTreeMap::<String, String>::new();
        let mm = MyMap(btm.clone());
        assert_eq!(format!("{:?}", mm), format!("{:?}", btm));
    }
    #[test]
    fn debug() {
        assert!(MyMap::<String, String>::new() == MyMap::<String, String>::new());
    }
    #[test]
    fn dupliacte() {
        let m = serde_yaml::from_str::<MyMap<String, String>>("x:\n\nx:");
        assert!(m.is_err());
        assert_eq!(
            format!("{:?}", m),
            "Err(Message(\"Element x is already existing\", Some(Pos { marker: Marker { index: 1, line: 1, col: 1 }, path: \".\" })))"
        );
    }
    #[test]
    fn unknown_format() {
        let input = "- A\n\n- B\n\n- C\n";
        let res = serde_yaml::from_str::<MyMap<String, String>>(input);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(Message(\"invalid type: sequence, expected a map with unique keys\", Some(Pos { marker: Marker { index: 0, line: 1, col: 0 }, path: \".\" })))"
        );
    }
}
