use reqwest::Url;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use url::{form_urlencoded, UrlQuery};

#[derive(Clone, Debug)]
pub struct BaseUrl(Url);

impl BaseUrl {
    pub fn new(url: Url) -> Result<Self, url::ParseError> {
        if url.cannot_be_a_base() {
            Err(url::ParseError::RelativeUrlWithCannotBeABaseBase)
        } else {
            Ok(Self(url))
        }
    }

    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        BaseUrl::new(Url::parse(input)?)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_url(self) -> Url {
        self.0
    }

    pub fn extend<I>(&mut self, segments: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.0
            .path_segments_mut()
            .expect("wrapped URL can be a base")
            .pop_if_empty()
            .extend(segments);
    }

    pub fn query_pairs_mut(&mut self) -> form_urlencoded::Serializer<'_, UrlQuery<'_>> {
        self.0.query_pairs_mut()
    }
}

impl AsRef<str> for BaseUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Url> for BaseUrl {
    fn as_ref(&self) -> &Url {
        &self.0
    }
}

impl From<BaseUrl> for Url {
    fn from(base_url: BaseUrl) -> Self {
        base_url.into_url()
    }
}

impl<'de> Deserialize<'de> for BaseUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BaseUrlVisitor)
    }
}

struct BaseUrlVisitor;

impl<'de> Visitor<'de> for BaseUrlVisitor {
    type Value = BaseUrl;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a base URL")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let create_err = || serde::de::Error::invalid_value(Unexpected::Str(s), &self);

        let url = Url::parse(s).map_err(|_| create_err())?;
        BaseUrl::new(url).map_err(|_| create_err())
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use super::BaseUrl;

    #[test]
    fn test_extend_base_url_without_trailing_slash() {
        let mut url = BaseUrl::parse("http://localhost/foo").unwrap();
        url.extend(["bar"]);
        assert_eq!(url.as_str(), "http://localhost/foo/bar");
    }

    #[test]
    fn test_extend_base_url_with_trailing_slash() {
        let mut url = BaseUrl::parse("http://localhost/foo/").unwrap();
        url.extend(["bar"]);
        assert_eq!(url.as_str(), "http://localhost/foo/bar");
    }

    #[test]
    fn test_parsing_non_base_url_returns_error() {
        let result = BaseUrl::parse("/foo/");
        assert!(result.is_err());

        let result = BaseUrl::parse("data:text/plain,foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_non_base_url_returns_error() {
        let result = BaseUrl::new(Url::parse("data:text/plain,foo").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_query_params() {
        let mut url = BaseUrl::parse("http://localhost/foo").unwrap();
        url.query_pairs_mut().append_pair("q", "bar");
        assert_eq!(url.as_str(), "http://localhost/foo?q=bar");
    }
}
