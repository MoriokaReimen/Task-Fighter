use anyhow::Result;
use domain::Relation;
use duckdb::Row;
use duckdb::ToSql;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuckdbRelation {
    pub parent_id: i32,
    pub child_id: i32,
}

impl From<Relation> for DuckdbRelation {
    fn from(relation: Relation) -> Self {
        Self {
            parent_id: relation.parent_id,
            child_id: relation.child_id,
        }
    }
}

impl From<DuckdbRelation> for Relation {
    fn from(duckdb_relation: DuckdbRelation) -> Self {
        Self {
            parent_id: duckdb_relation.parent_id,
            child_id: duckdb_relation.child_id,
        }
    }
}

impl TryFrom<&Row<'_>> for DuckdbRelation {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            parent_id: row.get("parent_id")?,
            child_id: row.get("child_id")?,
        })
    }
}

impl DuckdbRelation {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            ("parent_id", &self.parent_id as &dyn ToSql),
            ("child_id", &self.child_id as &dyn ToSql),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    #[test]
    fn test_from_relation_into_duckdb_relation() {
        // 1. Relation <-> DuckdbRelation の相互変換（From トレイト）のテスト
        let domain_relation = Relation {
            parent_id: 10,
            child_id: 20,
        };

        // Relation -> DuckdbRelation
        let duckdb_relation = DuckdbRelation::from(domain_relation.clone());
        assert_eq!(duckdb_relation.parent_id, 10);
        assert_eq!(duckdb_relation.child_id, 20);

        // DuckdbRelation -> Relation
        let converted_domain = Relation::from(duckdb_relation);
        assert_eq!(converted_domain.parent_id, domain_relation.parent_id);
        assert_eq!(converted_domain.child_id, domain_relation.child_id);
    }

    #[test]
    fn test_to_named_params() {
        // 2. to_named_params のテスト（HashMap の保持チェックとキー削除）
        let duckdb_relation = DuckdbRelation {
            parent_id: 100,
            child_id: 200,
        };

        let mut params = duckdb_relation.to_named_params();

        // 必要なキーが過不足なく存在することを確認
        assert_eq!(params.len(), 2);
        assert!(params.contains_key("parent_id"));
        assert!(params.contains_key("child_id"));

        // HashMap からキーを削除する挙動の確認
        let removed_parent = params.remove("parent_id");
        assert!(removed_parent.is_some());
        assert!(!params.contains_key("parent_id"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        // 3. DBの Row からのパーステスト
        let conn = Connection::open_in_memory()?;

        // 一時的なテーブルを作成してテストデータを挿入
        conn.execute(
            "CREATE TABLE temp_relation (parent_id INT, child_id INT);",
            [],
        )?;
        conn.execute("INSERT INTO temp_relation VALUES (5, 8);", [])?;

        let mut stmt =
            conn.prepare("SELECT parent_id, child_id FROM temp_relation WHERE parent_id = 5;")?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // 参照の重複を防ぐため、row をそのまま渡す
        let parsed_relation = DuckdbRelation::try_from(row)?;

        assert_eq!(parsed_relation.parent_id, 5);
        assert_eq!(parsed_relation.child_id, 8);

        Ok(())
    }
}
