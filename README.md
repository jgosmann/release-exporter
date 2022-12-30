# release-exporter

Retrieves release information
and exports related metrics.

## Metrics

Currently,
a single metric `upgrades`
containing information about available upgrades
is supported.
This information is represented in the labels.
The metric value itself will be `1`.

The metric is configured
with the `upgrade_pending_checks` configuration key
(see Configuration section below).

The following labels will always exist:

* `name`:
  the name given in the `upgrade_pending_checks` configuration.
* `status`
  with the value being one of
  `unknown`,
  `upgrades-available`,
  `up-to-date`:
  indicates whether an upgrade is available.
  
Additionally,
all labels
of the release provider,
referenced in the `current` current field of the `upgrades_pending_checks` configuration,
will be added.


## Configuration

The configuration has to be in YAML format.
It uses two main keys:

* `providers` (list)
  to configure
  a number of release providers
  that provide information
  about releases available and in use,
* `upgrade_pending_checks` (list)
  to configure
  which release versions to compare
  to determine available upgrades.
  
An example configuration can be found in `sample-conf.yml`.

### Providers

Each provider must have at least the following two keys:

* `name` (string):
  a unique name used to refer to this configured provider,
* `provider`
  (enum): the provider type to fetch releases with.

Each provider type has additional required and optional keys.

A provider returns a set of releases
where each release has different labels.

#### latest_github_release provider

Retrieves the latest release from a Github repository.

Accepts the following configuration keys:

* `repo` (string):
  the repository in the form `username/repo`.
* `tag_name_regex` (string, default `^v?(.*)$`):
  a regular expression
  to extract the version number from the release tag.
  Uses the [syntax of Rust's regex crate][regex-syntax].
* `version_fmt` (string, default `${1}`):
  an expression to construct the version
  from the capture groups of `tag_name_regex`.
* `api_url` (string, default: `https://api.github.com`):
  the URL of the Github API.
  
##### prometheus provider

Retrieves versions from a Prometheus metric label.

* `query` (string):
  Prometheus query
  to retrieve the metric with the version.
  It may return multiple versions with different labels.
  All labels,
  except for one given with `label`,
  will be attached to the release.
* `label` (string, default: `version`):
  the label containing the version information.
* `api_url` (string, default: `http://localhost:9090/api`):
  the URL of the Prometheus API.
  

### upgrade_pending_checks
  
Configures the check between release versions
to determine available upgrades.
These checks are exported as the `upgrades` metric.
Each item accepts the following configuration keys:

* `name` (string):
  name of the check.
  Will be used as the `name` label
  in the `upgrades` metric.
* `current` (string, default: `current_{name}_release`):
  must refer to a provider name.
  That provider is used to determine the current version in use.
* `latest` (string, default: `latest_{name}_release`):
  must refer to a provider name.
  That provider is used to deterimne the latest available version.

Note the handling of labels:

* All labels
  obtained from the `current` provider
  will be replicated in the output metric.
* Each release provided by `current` will be attempted to match
  with a release provided by `latest`.
  To be considered matching,
  all labels of the `latest` release must be present
  and have the same value
  as in the `current` release.


## Usage

```
Usage: release-exporter [OPTIONS] --config.file <CONFIG>

Options:
      --config.file <CONFIG>
          Configuration file to load
      --http.timout <HTTP_TIMEOUT_SECONDS>
          Timeout for HTTP requests (seconds) [default: 10]
      --web.listen-address <LISTEN_ADDRESS>
          Address on which to expose metrics [default: localhost:31343]
  -h, --help
          Print help information
  -V, --version
          Print version information
```

[regex-syntax]: https://docs.rs/regex/latest/regex/#syntax
