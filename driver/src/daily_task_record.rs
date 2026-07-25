use crate::duckdb_daily_task::DuckdbDailyTask;
use anyhow::{Context, Result, bail};
use domain::DailyTask;
use domain::{DailyTaskFilterFlags, DailyTaskOrderFlags, DailyTaskSearchFlags};
use duckdb::Connection;
use tracing::info;

pub fn insert_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    const INSERT_TASK_SQL: &str = include_str!("../assets/daily_task_sql/insert_daily_task.sql");

    info!("Inserting daily_task: {:?}", daily_task);
    let duckdb_daily_task: DuckdbDailyTask = daily_task.clone().into();
    let mut params = duckdb_daily_task.to_named_params();
    params.remove("id");
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    let _ = stmt.query(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invalid id: {id}"));
    }

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM daily_tasks WHERE id = ?;",
            duckdb::params![id],
            |row| row.get(0),
        )
        .context("Failed to check if daily_task ID exists")?;

    Ok(count > 0)
}

pub fn upsert_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    let exists = exists_id(conn, daily_task.id)?;
    if exists {
        info!("ID {} exists. Update daily_task.", daily_task.id);
        update_daily_task(conn, daily_task)?;
    } else {
        info!("ID {} not exists. Create new daily_task.", daily_task.id);
        insert_daily_task(conn, daily_task)?;
    }

    Ok(())
}

pub fn get_next_daily_task_id(conn: &Connection) -> Result<i32> {
    const GET_NEXT_DAILY_TASK_ID_SQL: &str =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'daily_tasks_id_seq';";
    let mut stmt = conn.prepare(GET_NEXT_DAILY_TASK_ID_SQL)?;
    let last_value: Option<i64> = stmt
        .query_row([], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map_or(1, |val| (val + 1) as i32);
    info!("Next daily_task id is {}.", next_id);

    Ok(next_id)
}

pub fn update_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    const UPDATE_TASK_SQL: &str = include_str!("../assets/daily_task_sql/update_daily_task.sql");

    info!("Updating daily_task: {:?}", daily_task);
    let duckdb_daily_task: DuckdbDailyTask = daily_task.clone().into();

    let params = duckdb_daily_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("DailyTask {} update success.", daily_task.id);
    Ok(())
}

pub fn search_daily_task(
    conn: &Connection,
    pattern: &str,
    search_flags: DailyTaskSearchFlags,
    filter_flags: DailyTaskFilterFlags,
    order_flags: DailyTaskOrderFlags,
) -> Result<Vec<DailyTask>> {
    const SEARCH_SQL: &str = include_str!("../assets/daily_task_sql/search_daily_task.sql");

    info!("Searching daily_tasks with pattern: '{}'", pattern);
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_daily_tasks = stmt
        .query_map(params, |row| DuckdbDailyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbDailyTask>, duckdb::Error>>()?;

    let daily_tasks = duckdb_daily_tasks
        .into_iter()
        .map(DailyTask::try_from)
        .collect::<Result<Vec<DailyTask>>>()?;

    Ok(daily_tasks)
}

pub fn fetch_one_daily_task(conn: &Connection, id: i32) -> Result<DailyTask> {
    const FETCH_ONE_SQL: &str = include_str!("../assets/daily_task_sql/fetch_one_daily_task.sql");

    info!("Querying daily_task with id: {}", id);
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_daily_task = stmt.query_row(duckdb::named_params! { "id": id }, |row| {
        DuckdbDailyTask::try_from(row)
    })?;

    let daily_task = DailyTask::try_from(duckdb_daily_task)?;
    Ok(daily_task)
}

pub fn fetch_all_daily_task(
    conn: &Connection,
    filter_flags: DailyTaskFilterFlags,
    order_flags: DailyTaskOrderFlags,
) -> Result<Vec<DailyTask>> {
    const FETCH_ALL_SQL: &str = include_str!("../assets/daily_task_sql/fetch_all_daily_task.sql");

    info!("Querying daily_tasks");
    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;
    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_daily_tasks = stmt
        .query_map(params, |row| DuckdbDailyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbDailyTask>, duckdb::Error>>()?;

    let daily_tasks = duckdb_daily_tasks
        .into_iter()
        .map(DailyTask::try_from)
        .collect::<Result<Vec<DailyTask>>>()?;

    Ok(daily_tasks)
}

pub fn delete_daily_task(conn: &Connection, id: i32) -> Result<()> {
    const DELETE_SQL: &str = include_str!("../assets/daily_task_sql/delete_daily_task.sql");
    info!("Delete daily task: {}", id);
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(duckdb::named_params! { "id": id })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DuckdbPath;
    use crate::connect;
    use domain::*;
    use duckdb::Connection;

    fn setup_in_memory_db() -> Connection {
        let path = DuckdbPath::InMemory;
        let conn = connect(&path);

        conn.unwrap()
    }

    // テスト用のダミーDailyTaskデータを生成するヘルパー関数
    fn create_dummy_task(id: i32, title: &str) -> DailyTask {
        DailyTask {
            id,
            active: true,
            project: "TestProject".to_string(),
            title: title.to_string(),
            detail: "TestDetail".to_string(),
            priority: TaskPriority::Low,
        }
    }

    // 各ビットフラグのダミーインスタンス（環境に応じて空のフラグに差し替えてください）
    fn dummy_flags() -> (
        DailyTaskSearchFlags,
        DailyTaskFilterFlags,
        DailyTaskOrderFlags,
    ) {
        // 例として bits から復元するか、Default等があればそちらを使用してください
        (
            DailyTaskSearchFlags::default(),
            DailyTaskFilterFlags::default(),
            DailyTaskOrderFlags::default(),
        )
    }

    #[test]
    fn test_insert_and_fetch_one_daily_task() -> Result<()> {
        let conn = setup_in_memory_db();
        let task = create_dummy_task(1, "Insert Test");

        // 挿入テスト
        insert_daily_task(&conn, &task)?;

        // 取得テスト
        let fetched = fetch_one_daily_task(&conn, 1)?;
        assert_eq!(fetched.id, task.id);
        assert_eq!(fetched.title, task.title);

        Ok(())
    }

    #[test]
    fn test_exists_id_validation() {
        let conn = setup_in_memory_db();

        // 異常系: id が 0 以下のときは bail! でエラーになるか確認
        let result = exists_id(&conn, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid id: 0"));
    }

    #[test]
    fn test_upsert_daily_task() -> Result<()> {
        let conn = setup_in_memory_db();
        let task = create_dummy_task(1, "Initial Title");

        // 1回目: まだ存在しないので Insert されるはず
        upsert_daily_task(&conn, &task)?;
        let fetched_inserted = fetch_one_daily_task(&conn, 1)?;
        assert_eq!(fetched_inserted.title, "Initial Title");

        // 2回目: 存在するので Update されるはず
        let mut updated_task = task;
        updated_task.title = "Upserted Title".to_string();
        upsert_daily_task(&conn, &updated_task)?;

        let fetched_updated = fetch_one_daily_task(&conn, 1)?;
        assert_eq!(fetched_updated.title, "Upserted Title");

        Ok(())
    }

    #[test]
    fn test_get_next_daily_task_id() -> Result<()> {
        let conn = setup_in_memory_db();

        // 最初はシーケンスが利用前（または1）なので初期値を期待
        let next_id = get_next_daily_task_id(&conn)?;
        // 既存実装の last_value.map(...).unwrap_or(1) に従う
        assert_eq!(next_id, 1);

        // 実際にシーケンスを進めるダミークエリを発行
        conn.execute("SELECT nextval('daily_tasks_id_seq');", [])?;

        // シーケンスが進んだ後の次ID取得テスト
        let next_id_after = get_next_daily_task_id(&conn)?;
        assert_eq!(next_id_after, 2);

        Ok(())
    }

    #[test]
    fn test_fetch_all_daily_task() -> Result<()> {
        let conn = setup_in_memory_db();
        let task1 = create_dummy_task(1, "Task 1");
        let task2 = create_dummy_task(2, "Task 2");

        insert_daily_task(&conn, &task1)?;
        insert_daily_task(&conn, &task2)?;

        let (_, filter_flags, order_flags) = dummy_flags();
        let list = fetch_all_daily_task(&conn, filter_flags, order_flags)?;

        assert_eq!(list.len(), 2);
        Ok(())
    }

    #[test]
    fn test_search_daily_task() -> Result<()> {
        let conn = setup_in_memory_db();
        let task1 = create_dummy_task(1, "Apple Task");
        let task2 = create_dummy_task(2, "Banana Task");

        insert_daily_task(&conn, &task1)?;
        insert_daily_task(&conn, &task2)?;

        let (search_flags, filter_flags, order_flags) = dummy_flags();

        let search_results =
            search_daily_task(&conn, "Apple", search_flags, filter_flags, order_flags)?;

        assert!(search_results.iter().any(|t| t.title.contains("Apple")));

        Ok(())
    }

    #[test]
    fn test_delete_daily_task() -> Result<()> {
        let conn = setup_in_memory_db();
        let task = create_dummy_task(99, "To Be Deleted");

        insert_daily_task(&conn, &task)?;
        assert!(fetch_one_daily_task(&conn, 1).is_ok());

        delete_daily_task(&conn, 1)?;

        let result = fetch_one_daily_task(&conn, 1);
        assert!(result.is_ok());
        if let Ok(daily_task) = result {
            assert!(!daily_task.active);
        }

        Ok(())
    }
}
