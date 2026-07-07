use crate::duckdb_relation::DuckdbRelation;
use anyhow::Result;
use domain::Relation;
use duckdb::Connection;
use tracing::info;

pub fn add_relation(conn: &Connection, relation: &Relation) -> Result<()> {
    info!("Inserting relation: {:?}", relation);
    const ADD_PARENT_SQL: &str = include_str!("../assets/relation_sql/add_parent.sql");
    let duckdb_relation: DuckdbRelation = relation.clone().into();
    let params = duckdb_relation.to_named_params();
    let mut stmt = conn.prepare(ADD_PARENT_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn get_parents(conn: &Connection, task_id: i32) -> Result<Vec<Relation>> {
    info!("Getting all parents of: {:?}", task_id);
    const GET_PARENT_SQL: &str = include_str!("../assets/relation_sql/get_parents.sql");
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
    info!("Getting all children of: {:?}", task_id);
    const GET_CHILDREN_SQL: &str = include_str!("../assets/relation_sql/get_children.sql");
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
    info!("Getting all relatives of: {:?}", task_id);
    const GET_RELATIVES_SQL: &str = include_str!("../assets/relation_sql/get_relatives.sql");
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
    info!("Deleting relation: {:?}", relation);
    const DELETE_SQL: &str = include_str!("../assets/relation_sql/delete_relation.sql");
    let duckdb_relation: DuckdbRelation = relation.clone().into();
    let params = duckdb_relation.to_named_params();
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}
