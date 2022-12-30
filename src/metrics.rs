use std::fmt::Write;

use prometheus_client::{
    encoding::{EncodeLabel, EncodeLabelSet, EncodeLabelValue, LabelSetEncoder},
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

use crate::checks::upgrade_pending::{CheckStatus, LabeledStatus};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct UpgradeLabels {
    status: CheckStatus,
    name: String,
    latest_version: Option<String>,
    additional_labels: Vec<(String, String)>,
}

impl EncodeLabelValue for CheckStatus {
    fn encode(
        &self,
        encoder: &mut prometheus_client::encoding::LabelValueEncoder,
    ) -> Result<(), std::fmt::Error> {
        encoder.write_str(match self {
            CheckStatus::Unknown => "unknown",
            CheckStatus::UpToDate => "up-to-date",
            CheckStatus::UpgradeAvailable => "upgrade-available",
        })
    }
}

impl EncodeLabelSet for UpgradeLabels {
    fn encode(&self, mut encoder: LabelSetEncoder) -> Result<(), std::fmt::Error> {
        ("status", self.status).encode(encoder.encode_label())?;
        ("name", self.name.as_str()).encode(encoder.encode_label())?;
        if let Some(latest_version) = &self.latest_version {
            ("latest_version", latest_version.as_str()).encode(encoder.encode_label())?;
        }
        for label in &self.additional_labels {
            (label.0.as_str(), label.1.as_str()).encode(encoder.encode_label())?;
        }
        Ok(())
    }
}

pub struct Metrics {
    upgrades: Family<UpgradeLabels, Gauge>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            upgrades: Family::default(),
        }
    }

    pub fn update<'a, I>(&self, check_results: I)
    where
        I: Iterator<Item = (&'a str, Vec<LabeledStatus<'a>>)>,
    {
        self.upgrades.clear();
        for (name, releases) in check_results {
            for release in releases {
                self.upgrades
                    .get_or_create(&UpgradeLabels {
                        name: name.into(),
                        status: release.status,
                        latest_version: release.latest_version.map(String::from),
                        additional_labels: release
                            .labels
                            .iter()
                            .map(|item| (item.0.clone(), item.1.clone()))
                            .collect(),
                    })
                    .set(1);
            }
        }
    }

    pub fn register(&self, registry: &mut Registry) {
        registry.register(
            "upgrades",
            "Count of different upgrade states (unknown, up-to-date, upgrade-available)",
            self.upgrades.clone(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use prometheus_client::{encoding::text::encode, registry::Registry};

    use crate::checks::upgrade_pending::{CheckStatus, LabeledStatus};

    use super::Metrics;

    #[test]
    fn test_update_metrics() {
        let metrics = Metrics::new();
        let labels = HashMap::from([("label".into(), "label-value".into())]);

        let check_results = [(
            "check_name",
            vec![LabeledStatus {
                labels: &labels,
                status: CheckStatus::UpToDate,
                latest_version: "current-version".into(),
            }],
        )];
        metrics.update(check_results.into_iter());

        let check_results = [(
            "check_name",
            vec![LabeledStatus {
                labels: &labels,
                status: CheckStatus::UpgradeAvailable,
                latest_version: "latest-version".into(),
            }],
        )];
        metrics.update(check_results.into_iter());

        let mut registry = <Registry>::default();
        metrics.register(&mut registry);
        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        let buffer: String = buffer
            .split('\n')
            .filter(|line| !line.starts_with('#'))
            .collect();
        assert_eq!(
            buffer,
            "upgrades{status=\"upgrade-available\",name=\"check_name\",latest_version=\"latest-version\",label=\"label-value\"} 1"
        );
    }
}
