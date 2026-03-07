//! Conservation score computation and reporting.

use super::audit::AuditLog;
use super::tokens::TokenMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationReport {
    pub score: f64,
    pub total_tokens: u64,
    pub total_savings: u64,
    pub cache_hit_rate: f64,
    pub layer_breakdown: LayerBreakdown,
    pub verdict: ConservationVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerBreakdown {
    pub cache_tokens: u64,
    pub index_tokens: u64,
    pub scoped_tokens: u64,
    pub delta_tokens: u64,
    pub full_tokens: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConservationVerdict {
    Excellent, // >= 0.8
    Good,      // >= 0.6
    Fair,      // >= 0.4
    Poor,      // >= 0.2
    Wasteful,  // < 0.2
}

impl ConservationVerdict {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::Excellent
        } else if score >= 0.6 {
            Self::Good
        } else if score >= 0.4 {
            Self::Fair
        } else if score >= 0.2 {
            Self::Poor
        } else {
            Self::Wasteful
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Excellent => "excellent",
            Self::Good => "good",
            Self::Fair => "fair",
            Self::Poor => "poor",
            Self::Wasteful => "wasteful",
        }
    }
}

pub fn generate_report(metrics: &TokenMetrics, audit: &AuditLog) -> ConservationReport {
    let score = metrics.conservation_score();
    let total_tokens = metrics.total_tokens();
    let total_savings = metrics.total_savings();
    let cache_hit_rate = audit.cache_hit_rate();

    use std::sync::atomic::Ordering;
    let layer_breakdown = LayerBreakdown {
        cache_tokens: metrics.layer0_cache.load(Ordering::Relaxed),
        index_tokens: metrics.layer1_index.load(Ordering::Relaxed),
        scoped_tokens: metrics.layer2_scoped.load(Ordering::Relaxed),
        delta_tokens: metrics.layer3_delta.load(Ordering::Relaxed),
        full_tokens: metrics.layer4_full.load(Ordering::Relaxed),
    };

    ConservationReport {
        score,
        total_tokens,
        total_savings,
        cache_hit_rate,
        layer_breakdown,
        verdict: ConservationVerdict::from_score(score),
    }
}

#[cfg(test)]
mod tests {
    use super::super::tokens::Layer;
    use super::*;

    #[test]
    fn test_verdict_from_score() {
        assert_eq!(
            ConservationVerdict::from_score(0.9),
            ConservationVerdict::Excellent
        );
        assert_eq!(
            ConservationVerdict::from_score(0.7),
            ConservationVerdict::Good
        );
        assert_eq!(
            ConservationVerdict::from_score(0.5),
            ConservationVerdict::Fair
        );
        assert_eq!(
            ConservationVerdict::from_score(0.3),
            ConservationVerdict::Poor
        );
        assert_eq!(
            ConservationVerdict::from_score(0.1),
            ConservationVerdict::Wasteful
        );
    }

    #[test]
    fn test_generate_report() {
        let metrics = TokenMetrics::new();
        let audit = AuditLog::new(100);

        // Simulate usage
        metrics.record(Layer::Full, 500, 500);
        metrics.record(Layer::Cache, 0, 500);
        metrics.record(Layer::Scoped, 50, 500);

        let report = generate_report(&metrics, &audit);
        assert!(report.score > 0.0);
        assert_eq!(report.total_tokens, 550);
        assert!(report.total_savings > 0);
    }

    #[test]
    fn test_report_with_warm_cache() {
        let metrics = TokenMetrics::new();
        let audit = AuditLog::new(100);

        // Cold: 5 full queries
        for _ in 0..5 {
            metrics.record(Layer::Full, 500, 500);
        }

        // Warm: 15 cache hits
        for _ in 0..15 {
            metrics.record(Layer::Cache, 0, 500);
        }

        let report = generate_report(&metrics, &audit);
        assert!(
            report.score >= 0.7,
            "Score should be >=0.7 with 75% cache hits: {}",
            report.score
        );
        assert!(
            report.verdict == ConservationVerdict::Good
                || report.verdict == ConservationVerdict::Excellent
        );
    }

    #[test]
    fn test_conservation_after_warmup() {
        let metrics = TokenMetrics::new();
        let audit = AuditLog::new(100);

        // Phase 1: All cold
        for _ in 0..10 {
            metrics.record(Layer::Full, 500, 500);
        }
        let cold_report = generate_report(&metrics, &audit);

        // Phase 2: All cached
        for _ in 0..10 {
            metrics.record(Layer::Cache, 0, 500);
        }
        let warm_report = generate_report(&metrics, &audit);

        assert!(
            warm_report.score > cold_report.score,
            "Warm score ({}) should exceed cold score ({})",
            warm_report.score,
            cold_report.score
        );
    }

    #[test]
    fn test_verdict_name() {
        assert_eq!(ConservationVerdict::Excellent.name(), "excellent");
        assert_eq!(ConservationVerdict::Wasteful.name(), "wasteful");
    }
}
