use std::collections::HashMap;

use serde::Deserialize;

use crate::providers::VersionInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradePendingCheck {
    pub name: String,
    pub current: String,
    pub latest: String,
}

#[derive(Clone, Debug, Deserialize)]
struct UpgradePendingCheckWithOptionals {
    name: String,
    current: Option<String>,
    latest: Option<String>,
}

impl<'de> Deserialize<'de> for UpgradePendingCheck {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpgradePendingCheckWithOptionals {
            name,
            current,
            latest,
        } = UpgradePendingCheckWithOptionals::deserialize(deserializer)?;
        Ok(Self {
            current: current.unwrap_or_else(|| format!("current_{name}_release")),
            latest: latest.unwrap_or_else(|| format!("latest_{name}_release")),
            name,
        })
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckStatus {
    UpToDate,
    UpgradeAvailable,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledStatus<'a> {
    pub labels: &'a HashMap<String, String>,
    pub status: CheckStatus,
    pub latest_version: Option<&'a str>,
}

impl UpgradePendingCheck {
    pub fn check<'a>(
        &self,
        releases: &'a HashMap<String, Vec<VersionInfo>>,
    ) -> Vec<LabeledStatus<'a>> {
        let current = releases.get(&self.current);
        let latest = releases.get(&self.latest);
        match (current, latest) {
            (None, _) => vec![],
            (Some(current), latest) => current
                .iter()
                .map(|v| {
                    let latest_version = Self::latest_version(v, latest.map(Vec::as_ref));
                    LabeledStatus {
                        labels: &v.labels,
                        status: match (&v.version, latest_version) {
                            (None, _) => CheckStatus::Unknown,
                            (_, None) => CheckStatus::Unknown,
                            (Some(current_version), Some(latest_version)) => {
                                if current_version == latest_version {
                                    CheckStatus::UpToDate
                                } else {
                                    CheckStatus::UpgradeAvailable
                                }
                            }
                        },
                        latest_version,
                    }
                })
                .collect(),
        }
    }

    fn latest_version<'a>(
        current: &VersionInfo,
        latest: Option<&'a [VersionInfo]>,
    ) -> Option<&'a str> {
        match latest {
            None => None,
            Some(latest) => {
                let matching_release = latest.iter().find(|v| {
                    v.labels
                        .iter()
                        .all(|(label, value)| current.labels.get(label) == Some(value))
                });
                matching_release.and_then(|r| r.version.as_deref())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_test::{assert_de_tokens, Token};

    use crate::{
        checks::upgrade_pending::{CheckStatus, LabeledStatus},
        providers::VersionInfo,
    };

    use super::UpgradePendingCheck;

    #[test]
    fn test_upgrade_pending_check() {
        let check = UpgradePendingCheck {
            name: "name".into(),
            current: "current".into(),
            latest: "latest".into(),
        };
        let mut releases: HashMap<String, Vec<VersionInfo>> = HashMap::new();
        releases.insert(
            "current".into(),
            vec![
                VersionInfo {
                    labels: HashMap::from([
                        ("stream".into(), "stable".into()),
                        ("instance".into(), "production-01".into()),
                    ]),
                    version: Some("v-current".into()),
                },
                VersionInfo {
                    labels: HashMap::from([
                        ("stream".into(), "stable".into()),
                        ("instance".into(), "production-02".into()),
                    ]),
                    version: Some("v-current".into()),
                },
                VersionInfo {
                    labels: HashMap::from([
                        ("stream".into(), "testing".into()),
                        ("instance".into(), "staging".into()),
                    ]),
                    version: Some("v-testing-current".into()),
                },
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "no-latest".into())]),
                    version: Some("v-no-latest".into()),
                },
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "no-match".into())]),
                    version: Some("v-no-match".into()),
                },
                VersionInfo {
                    labels: HashMap::from([
                        ("stream".into(), "testing".into()),
                        ("instance".into(), "unreachable".into()),
                    ]),
                    version: None,
                },
            ],
        );
        releases.insert(
            "latest".into(),
            vec![
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "stable".into())]),
                    version: Some("v-latest".into()),
                },
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "testing".into())]),
                    version: Some("v-testing-current".into()),
                },
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "no-latest".into())]),
                    version: None,
                },
                VersionInfo {
                    labels: HashMap::from([("stream".into(), "ignored".into())]),
                    version: Some("v-ignored".into()),
                },
            ],
        );
        releases.insert(
            "ignored".into(),
            vec![VersionInfo {
                labels: HashMap::from([("stream".into(), "stable".into())]),
                version: Some("v-ignored".into()),
            }],
        );
        let labels_stable_01 = HashMap::from([
            ("stream".into(), "stable".into()),
            ("instance".into(), "production-01".into()),
        ]);
        let labels_stable_02 = HashMap::from([
            ("stream".into(), "stable".into()),
            ("instance".into(), "production-02".into()),
        ]);
        let labels_testing = HashMap::from([
            ("stream".into(), "testing".into()),
            ("instance".into(), "staging".into()),
        ]);
        let labels_no_latest = HashMap::from([("stream".into(), "no-latest".into())]);
        let labels_no_match = HashMap::from([("stream".into(), "no-match".into())]);
        let labels_unreachable = HashMap::from([
            ("stream".into(), "testing".into()),
            ("instance".into(), "unreachable".into()),
        ]);
        let expected = vec![
            LabeledStatus {
                labels: &labels_stable_01,
                status: CheckStatus::UpgradeAvailable,
                latest_version: Some("v-latest"),
            },
            LabeledStatus {
                labels: &labels_stable_02,
                status: CheckStatus::UpgradeAvailable,
                latest_version: Some("v-latest"),
            },
            LabeledStatus {
                labels: &labels_testing,
                status: CheckStatus::UpToDate,
                latest_version: Some("v-testing-current"),
            },
            LabeledStatus {
                labels: &labels_no_latest,
                status: CheckStatus::Unknown,
                latest_version: None,
            },
            LabeledStatus {
                labels: &labels_no_match,
                status: CheckStatus::Unknown,
                latest_version: None,
            },
            LabeledStatus {
                labels: &labels_unreachable,
                status: CheckStatus::Unknown,
                latest_version: Some("v-testing-current"),
            },
        ];
        assert_eq!(check.check(&releases), expected);
    }

    #[test]
    fn test_deserialize_upgrade_pending_check_all_filled() {
        let expected = UpgradePendingCheck {
            name: "name-value".into(),
            current: "current-value".into(),
            latest: "latest-value".into(),
        };
        assert_de_tokens(
            &expected,
            &[
                Token::Map { len: Some(3) },
                Token::Str("name"),
                Token::Str("name-value"),
                Token::Str("current"),
                Token::Some,
                Token::Str("current-value"),
                Token::Str("latest"),
                Token::Some,
                Token::Str("latest-value"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_deserialize_upgrade_pending_check_minimal() {
        let expected = UpgradePendingCheck {
            name: "name-value".into(),
            current: "current_name-value_release".into(),
            latest: "latest_name-value_release".into(),
        };
        assert_de_tokens(
            &expected,
            &[
                Token::Map { len: Some(1) },
                Token::Str("name"),
                Token::Str("name-value"),
                Token::MapEnd,
            ],
        );
    }
}
