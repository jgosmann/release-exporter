use reqwest::Url;
use serde::{
    de::{Unexpected, Visitor},
    Deserializer, Serializer,
};

pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(url.to_string().as_str())
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing an URL")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Url::parse(s).map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(s), &self))
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(UrlVisitor)
}
