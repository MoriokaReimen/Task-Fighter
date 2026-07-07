use anyhow::Result;
use domain::Relation;
use duckdb::Row;
use duckdb::ToSql;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbRelation {
    pub parent_id: i32,
    pub child_id: i32,
}

impl From<Relation> for DuckdbRelation {
    fn from(relation: Relation) -> Self {
        DuckdbRelation {
            parent_id: relation.parent_id,
            child_id: relation.child_id,
        }
    }
}

impl From<DuckdbRelation> for Relation {
    fn from(duckdb_relation: DuckdbRelation) -> Self {
        Relation {
            parent_id: duckdb_relation.parent_id,
            child_id: duckdb_relation.child_id,
        }
    }
}

impl TryFrom<&Row<'_>> for DuckdbRelation {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbRelation {
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
