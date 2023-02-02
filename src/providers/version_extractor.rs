use regex::Regex;
use serde::Deserialize;

fn default_version_regex() -> Regex {
    Regex::new(r"^v?(.*)$").unwrap()
}

fn default_version_fmt() -> String {
    "${1}".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionExtractor {
    #[serde(default = "default_version_regex", with = "serde_regex")]
    version_regex: Regex,

    #[serde(default = "default_version_fmt")]
    version_fmt: String,
}

impl Default for VersionExtractor {
    fn default() -> Self {
        Self {
            version_regex: default_version_regex(),
            version_fmt: default_version_fmt(),
        }
    }
}

impl VersionExtractor {
    pub fn extract(&self, version: &str) -> Option<String> {
        self.version_regex.find(version).map(|version_match| {
            self.version_regex
                .replace(
                    &version[version_match.start()..version_match.end()],
                    &self.version_fmt,
                )
                .into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::VersionExtractor;

    #[test]
    fn test_extract_version() {
        assert_eq!(
            VersionExtractor::default().extract("1.2.3").unwrap(),
            String::from("1.2.3")
        );
        assert_eq!(
            VersionExtractor::default().extract("v1.2.3").unwrap(),
            String::from("1.2.3")
        );
    }
}
