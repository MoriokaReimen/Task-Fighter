#[derive(Default)]
pub struct Relation {
    pub id: i32,
    pub parent_task: i32,
    pub child_task: i32,
}

impl Relation {
    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.parent_task > 0 && self.child_task > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 共通で使える有効なRelationのヘルパー
    fn valid_relation() -> Relation {
        Relation {
            id: 1,
            parent_task: 10,
            child_task: 11,
        }
    }

    #[test]
    fn test_default_impl() {
        let default_relation = Relation::default();
        assert_eq!(default_relation.id, 0);
        assert_eq!(default_relation.parent_task, 0);
        assert_eq!(default_relation.child_task, 0);
    }

    #[test]
    fn test_is_valid_success() {
        let relation = valid_relation();
        assert!(relation.is_valid());
    }

    #[test]
    fn test_is_valid_failures() {
        // 1. id が 0 (不正)
        let mut relation = valid_relation();
        relation.id = 0;
        assert!(!relation.is_valid());

        // 2. id が 負の数 (不正)
        let mut relation = valid_relation();
        relation.id = -5;
        assert!(!relation.is_valid());

        // 3. parent_task が 0 (不正)
        let mut relation = valid_relation();
        relation.parent_task = 0;
        assert!(!relation.is_valid());

        // 4. child_task が 0 (不正)
        let mut relation = valid_relation();
        relation.child_task = 0;
        assert!(!relation.is_valid());
    }

    #[test]
    fn test_is_valid_boundary_value() {
        // 境界値テスト: すべてが最小の有効値である 1 の場合
        let relation = Relation {
            id: 1,
            parent_task: 1,
            child_task: 1,
        };
        assert!(relation.is_valid());
    }
}
