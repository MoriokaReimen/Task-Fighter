use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Relation {
    pub parent_id: i32,
    pub child_id: i32,
}

impl Relation {
    pub fn is_valid(&self) -> bool {
        self.parent_id > 0 && self.child_id > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 共通で使える有効なRelationのヘルパー
    fn valid_relation() -> Relation {
        Relation {
            parent_id: 10,
            child_id: 11,
        }
    }

    #[test]
    fn test_default_impl() {
        let default_relation = Relation::default();
        assert_eq!(default_relation.parent_id, 0);
        assert_eq!(default_relation.child_id, 0);
    }

    #[test]
    fn test_is_valid_success() {
        let relation = valid_relation();
        assert!(relation.is_valid());
    }

    #[test]
    fn test_is_valid_failures() {
        // 1. parent_id が 0 (不正)
        let mut relation = valid_relation();
        relation.parent_id = 0;
        assert!(!relation.is_valid());

        // 2. child_id が 0 (不正)
        let mut relation = valid_relation();
        relation.child_id = 0;
        assert!(!relation.is_valid());
    }

    #[test]
    fn test_is_valid_boundary_value() {
        // 境界値テスト: すべてが最小の有効値である 1 の場合
        let relation = Relation {
            parent_id: 1,
            child_id: 1,
        };
        assert!(relation.is_valid());
    }
}
