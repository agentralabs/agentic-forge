//! Token budget enforcement.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_tokens: u64,
    pub used_tokens: u64,
}

impl TokenBudget {
    pub fn new(max_tokens: u64) -> Self {
        Self {
            max_tokens,
            used_tokens: 0,
        }
    }

    pub fn unlimited() -> Self {
        Self {
            max_tokens: u64::MAX,
            used_tokens: 0,
        }
    }

    pub fn remaining(&self) -> u64 {
        self.max_tokens.saturating_sub(self.used_tokens)
    }

    pub fn is_exhausted(&self) -> bool {
        self.used_tokens >= self.max_tokens
    }

    pub fn can_afford(&self, cost: u64) -> bool {
        self.remaining() >= cost
    }

    pub fn spend(&mut self, tokens: u64) -> bool {
        if self.can_afford(tokens) {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn force_spend(&mut self, tokens: u64) {
        self.used_tokens += tokens;
    }

    pub fn utilization(&self) -> f64 {
        if self.max_tokens == 0 || self.max_tokens == u64::MAX {
            return 0.0;
        }
        self.used_tokens as f64 / self.max_tokens as f64
    }
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::unlimited()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_new() {
        let b = TokenBudget::new(1000);
        assert_eq!(b.remaining(), 1000);
        assert!(!b.is_exhausted());
    }

    #[test]
    fn test_budget_spend() {
        let mut b = TokenBudget::new(100);
        assert!(b.spend(50));
        assert_eq!(b.remaining(), 50);
        assert!(b.spend(50));
        assert!(b.is_exhausted());
        assert!(!b.spend(1));
    }

    #[test]
    fn test_budget_can_afford() {
        let b = TokenBudget::new(100);
        assert!(b.can_afford(100));
        assert!(!b.can_afford(101));
    }

    #[test]
    fn test_budget_unlimited() {
        let b = TokenBudget::unlimited();
        assert!(b.can_afford(u64::MAX - 1));
        assert!(!b.is_exhausted());
    }

    #[test]
    fn test_budget_utilization() {
        let mut b = TokenBudget::new(200);
        assert_eq!(b.utilization(), 0.0);
        b.spend(100);
        assert!((b.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_force_spend() {
        let mut b = TokenBudget::new(10);
        b.force_spend(100); // Over budget
        assert!(b.is_exhausted());
        assert_eq!(b.used_tokens, 100);
    }
}
