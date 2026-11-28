//! Validation tests for the fixture dataset and its attempt identities.

use super::*;

fn plan() -> Plan {
    Plan {
        strategy: "keyword".to_string(),
        provider: "alpha".to_string(),
    }
}

fn history_row(id: Option<&str>) -> HistoryAttempt {
    HistoryAttempt {
        id: id.map(str::to_string),
        plan: plan(),
        outcome: OutcomeClass::Verified,
        latency_ms: 42,
        cost_usd: 0.0,
        endpoint: "https://news.example/nba".to_string(),
    }
}

fn dataset_with(history: Vec<HistoryAttempt>) -> Dataset {
    Dataset {
        version: DATASET_VERSION.to_string(),
        families: vec![QueryFamily {
            id: "nba".to_string(),
            query: "latest basketball scores".to_string(),
            domain: "sports".to_string(),
            baseline: plan(),
            history,
            attempts: vec![EvalAttempt {
                plan: plan(),
                latency_ms: 42,
                cost_usd: 0.0,
                discovered: Vec::new(),
            }],
            truth: Truth {
                verified_urls: Vec::new(),
            },
        }],
        expectations: Expectations {
            min_recall_gain: 0.0,
            decisions: Vec::new(),
        },
    }
}

#[test]
fn redaction_drops_userinfo_query_and_fragment() {
    let endpoint =
        RedactedEndpoint::parse("https://reader:secret@news.example/nba/scores?utm=x#top")
            .expect("endpoint should parse");
    assert_eq!(endpoint.as_str(), "https://news.example/nba/scores");
    assert_eq!(endpoint.host(), "news.example");
}

#[test]
fn redaction_strips_query_only_urls() {
    let endpoint = RedactedEndpoint::parse("http://alpha.example/search?api_key=abc&q=rag")
        .expect("endpoint should parse");
    assert_eq!(endpoint.as_str(), "http://alpha.example/search");
    assert!(!endpoint.as_str().contains("api_key"));
}

#[test]
fn redaction_rejects_relative_and_foreign_schemes() {
    assert!(RedactedEndpoint::parse("news.example/nba").is_err());
    assert!(RedactedEndpoint::parse("ftp://news.example/nba").is_err());
    assert!(RedactedEndpoint::parse("https:///nba").is_err());
}

#[test]
fn outcome_classes_round_trip_through_their_tag_form() {
    for outcome in [
        OutcomeClass::Empty,
        OutcomeClass::Blocked,
        OutcomeClass::TransportError,
        OutcomeClass::Irrelevant,
        OutcomeClass::Useful,
        OutcomeClass::Verified,
    ] {
        assert_eq!(OutcomeClass::from_tag(outcome.as_str()), Some(outcome));
    }
    assert!(OutcomeClass::Verified.is_useful_signal());
    assert!(!OutcomeClass::Irrelevant.is_useful_signal());
    assert!(OutcomeClass::Blocked.is_blocking());
}

#[test]
fn attempt_identities_prefer_the_fixture_id_and_fall_back_to_the_row() {
    assert_eq!(
        history_row(Some("Nightly-Run-1")).identity("nba", 3),
        "nba-nightly-run-1"
    );
    assert_eq!(history_row(None).identity("nba", 3), "nba-h3");
}

#[test]
fn duplicate_attempt_identities_are_rejected() {
    let dataset = dataset_with(vec![history_row(Some("same")), history_row(Some("same"))]);
    let error = dataset
        .validate()
        .expect_err("a duplicate identity must fail loudly");
    assert!(
        error
            .to_string()
            .contains("duplicate history attempt identity")
    );
}

#[test]
fn attempt_identities_that_cannot_be_tags_are_rejected() {
    let dataset = dataset_with(vec![history_row(Some("has space"))]);
    let error = dataset
        .validate()
        .expect_err("an identity that cannot be a tag must fail loudly");
    assert!(error.to_string().contains("not a usable"));
}
