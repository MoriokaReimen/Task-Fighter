use crate::duckdb_relation::DuckdbRelation;
use anyhow::Result;
use domain::Relation;
use duckdb::Connection;
use tracing::info;

pub fn add_relation(conn: &Connection, relation: &Relation) -> Result<()> {
    const ADD_PARENT_SQL: &str = include_str!("../assets/relation_sql/add_parent.sql");
    info!("Inserting relation: {:?}", relation);
    let duckdb_relation: DuckdbRelation = relation.clone().into();
    let params = duckdb_relation.to_named_params();
    let mut stmt = conn.prepare(ADD_PARENT_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn get_parents(conn: &Connection, task_id: i32) -> Result<Vec<Relation>> {
    const GET_PARENT_SQL: &str = include_str!("../assets/relation_sql/get_parents.sql");
    info!("Getting all parents of: {:?}", task_id);
    let params = duckdb::named_params! { "child_id": task_id };
    let mut stmt = conn.prepare(GET_PARENT_SQL)?;
    let duckdb_relation = stmt
        .query_map(params, |row| DuckdbRelation::try_from(row))?
        .collect::<Result<Vec<DuckdbRelation>, duckdb::Error>>()?;
    let relation = duckdb_relation
        .into_iter()
        .map(Relation::from)
        .collect::<Vec<Relation>>();

    Ok(relation)
}

pub fn get_children(conn: &Connection, task_id: i32) -> Result<Vec<Relation>> {
    const GET_CHILDREN_SQL: &str = include_str!("../assets/relation_sql/get_children.sql");
    info!("Getting all children of: {:?}", task_id);
    let params = duckdb::named_params! { "parent_id": task_id };
    let mut stmt = conn.prepare(GET_CHILDREN_SQL)?;
    let duckdb_relation = stmt
        .query_map(params, |row| DuckdbRelation::try_from(row))?
        .collect::<Result<Vec<DuckdbRelation>, duckdb::Error>>()?;
    let relation = duckdb_relation
        .into_iter()
        .map(Relation::from)
        .collect::<Vec<Relation>>();

    Ok(relation)
}

pub fn get_relatives(conn: &Connection, task_id: i32) -> Result<Vec<Relation>> {
    const GET_RELATIVES_SQL: &str = include_str!("../assets/relation_sql/get_relatives.sql");
    info!("Getting all relatives of: {:?}", task_id);
    let params = duckdb::named_params! { "task_id": task_id };
    let mut stmt = conn.prepare(GET_RELATIVES_SQL)?;
    let duckdb_relation = stmt
        .query_map(params, |row| DuckdbRelation::try_from(row))?
        .collect::<Result<Vec<DuckdbRelation>, duckdb::Error>>()?;
    let relation = duckdb_relation
        .into_iter()
        .map(Relation::from)
        .collect::<Vec<Relation>>();

    Ok(relation)
}

pub fn delete_relation(conn: &Connection, relation: &Relation) -> Result<()> {
    const DELETE_SQL: &str = include_str!("../assets/relation_sql/delete_relation.sql");
    info!("Deleting relation: {:?}", relation);
    let duckdb_relation: DuckdbRelation = relation.clone().into();
    let params = duckdb_relation.to_named_params();
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DuckdbPath, connect};
    use duckdb::Connection;

    fn setup_test_db() -> Result<Connection> {
        let path = DuckdbPath::InMemory;
        connect(&path)
    }

    fn create_mock_relation(parent_id: i32, child_id: i32) -> Relation {
        Relation {
            parent_id,
            child_id,
        }
    }

    #[test]
    fn test_add_and_get_parents() -> Result<()> {
        let conn = setup_test_db()?;
        let relation1 = create_mock_relation(1, 10);
        let relation2 = create_mock_relation(2, 10);

        // 1. リレーションの追加
        assert!(add_relation(&conn, &relation1).is_ok());
        assert!(add_relation(&conn, &relation2).is_ok());

        // 2. 指定した子(10)に対する親タスク(1, 2)の取得検証
        let parents = get_parents(&conn, 10).unwrap();
        assert_eq!(parents.len(), 2);

        // 取得したリレーションのIDが正しいか検証
        let parent_ids: Vec<i32> = parents.iter().map(|r| r.parent_id).collect();
        assert!(parent_ids.contains(&1));
        assert!(parent_ids.contains(&2));
        Ok(())
    }

    #[test]
    fn test_get_children() -> Result<()> {
        let conn = setup_test_db()?;
        let relation1 = create_mock_relation(1, 10);
        let relation2 = create_mock_relation(1, 11);

        add_relation(&conn, &relation1).unwrap();
        add_relation(&conn, &relation2).unwrap();

        // 指定した親(1)に対する子タスク(10, 11)の取得検証
        let children = get_children(&conn, 1).unwrap();
        assert_eq!(children.len(), 2);

        let child_ids: Vec<i32> = children.iter().map(|r| r.child_id).collect();
        assert!(child_ids.contains(&10));
        assert!(child_ids.contains(&11));
        Ok(())
    }

    #[test]
    fn test_get_relatives() -> Result<()> {
        let conn = setup_test_db()?;
        // タスク5 を中心に、親が3、子が7 という関係性を作る
        let parent_relation = create_mock_relation(3, 5);
        let child_relation = create_mock_relation(5, 7);

        add_relation(&conn, &parent_relation).unwrap();
        add_relation(&conn, &child_relation).unwrap();

        // タスク5 に関連する（親または子である）すべてのリレーションを取得
        let relatives = get_relatives(&conn, 5).unwrap();
        assert_eq!(relatives.len(), 2);

        Ok(())
    }

    #[test]
    fn test_delete_relation() -> Result<()> {
        let conn = setup_test_db()?;
        let relation = create_mock_relation(1, 10);

        // 追加して存在することを確認
        add_relation(&conn, &relation).unwrap();
        let children_before = get_children(&conn, 1).unwrap();
        assert_eq!(children_before.len(), 1);

        // 削除処理の検証
        assert!(delete_relation(&conn, &relation).is_ok());

        // 削除後は取得できないことを確認
        let children_after = get_children(&conn, 1).unwrap();
        assert!(children_after.is_empty());

        Ok(())
    }
}
