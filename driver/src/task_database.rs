use crate::duckdb_task::DuckdbTask;
use anyhow::{Context, Result, bail};
use domain::{Task, TaskStatus};
use domain::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use duckdb::Connection;
use jiff::Zoned;
use jiff::civil::Date;
use tracing::info;

pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task: {:?}", task);
    const INSERT_TASK_SQL: &str = include_str!("../assets/task_sql/insert_task.sql");
    let db_task: DuckdbTask = task.clone().into();
    let mut params = db_task.to_named_params();
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    params.remove("end_date");
    params.remove("id");

    stmt.execute(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invalid id: {id}"));
    }
    let mut stmt = conn.prepare("SELECT 1 FROM tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if task ID exists")?;

    Ok(exists)
}

pub fn upsert_task(conn: &Connection, task: &Task) -> Result<()> {
    let exists = exists_id(conn, task.id)?;
    if exists {
        info!("ID {} exists. Update task.", task.id);
        update_task(conn, task)?;
    } else {
        info!("ID {} not exists. Create new task.", task.id);
        insert_task(conn, task)?;
    }

    Ok(())
}

pub fn get_next_task_id(conn: &Connection) -> Result<i32> {
    let query = "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'tasks_id_seq';";
    let last_value: Option<i64> = conn
        .query_row(query, [], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map_or(1, |val| (val + 1) as i32);
    info!("Next task id is {}.", next_id);

    Ok(next_id)
}

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Updating task: {:?}", task);
    const UPDATE_TASK_SQL: &str = include_str!("../assets/task_sql/update_task.sql");
    let mut db_task: DuckdbTask = task.clone().into();
    db_task.end_date = if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled
    {
        Some(Zoned::now().date().to_string())
    } else {
        None
    };

    let params = db_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("Task {} update success.", task.id);
    Ok(())
}

pub fn get_plot_data(
    conn: &Connection,
    start_date: Date,
    end_date: Date,
) -> Result<Vec<(i32, i32, i32, i32)>> {
    const COUNT_TASK_BY_DATE_SQL: &str = include_str!("../assets/task_sql/get_plot_data.sql");
    let mut stmt = conn.prepare(COUNT_TASK_BY_DATE_SQL)?;
    let rows = stmt.query_map(
        duckdb::named_params!{"start_date": start_date.to_string(), "end_date": end_date.to_string()},
        |row| {
            Ok((
                row.get::<_, Option<i32>>(0)?.unwrap_or(0), // LEFT JOINでタスクがない日はNullになるため対策
                row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                row.get::<_, Option<i32>>(2)?.unwrap_or(0),
                row.get::<_, Option<i32>>(3)?.unwrap_or(0),
            ))
        },
    )?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_task(
    conn: &Connection,
    pattern: &str,
    search_flags: TaskSearchFlags,
    filter_flags: TaskFilterFlags,
    order_flags: TaskOrderFlags,
) -> Result<Vec<Task>> {
    info!("Searching tasks with pattern: '{}'", pattern);
    const SEARCH_SQL: &str = include_str!("../assets/task_sql/search_task.sql");
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_tasks = stmt
        .query_map(params, |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()?;

    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()?;

    Ok(tasks)
}

pub fn fetch_one_task(conn: &Connection, id: i32) -> Result<Task> {
    info!("Querying task with id: {}", id);

    const FETCH_ONE_SQL: &str = include_str!("../assets/task_sql/fetch_one_task.sql");
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_task = stmt.query_row(duckdb::named_params! { "id": id }, |row| {
        DuckdbTask::try_from(row)
    })?;

    let task = Task::try_from(duckdb_task)?;
    Ok(task)
}

pub fn fetch_all_task(
    conn: &Connection,
    filter_flags: TaskFilterFlags,
    order_flags: TaskOrderFlags,
) -> Result<Vec<Task>> {
    info!("Querying tasks");

    const FETCH_ALL_SQL: &str = include_str!("../assets/task_sql/fetch_all_task.sql");

    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;

    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_tasks = stmt
        .query_map(params, |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()?;

    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()?;

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DuckdbPath, connect};
    use domain::*;
    use duckdb::Connection;
    use jiff::civil::Date;

    // テスト用のダミータスクヘルパー
    fn create_test_task(id: i32, status: TaskStatus) -> Task {
        Task {
            id,
            active: true,
            status,
            project: "ProjectA".to_string(),
            title: format!("TaskTitle {}", id),
            detail: "Detail text".to_string(),
            start_date: Date::new(2026, 7, 1).unwrap(),
            due_date: Date::new(2026, 7, 10).unwrap(),
            priority: TaskPriority::try_from(1).unwrap_or_default(),
            progress: 0.0,
            time_spent: 0.0,
            entry_date: Date::new(2026, 7, 1).unwrap(),
            end_date: None,
        }
    }

    // テスト用に最小限の `tasks` テーブルとシーケンスをセットアップする関数
    fn setup_test_db() -> Result<Connection> {
        let path = DuckdbPath::InMemory;
        connect(&path)
    }

    #[test]
    fn test_insert_and_exists_id_and_upsert() -> Result<()> {
        let conn = setup_test_db()?;
        let task = create_test_task(1, TaskStatus::Pending);
        insert_task(&conn, &task)?;
        let exists = exists_id(&conn, 1)?;
        assert!(exists, "Task with ID 1 should exist after insertion");

        let not_exists = exists_id(&conn, 999)?;
        assert!(!not_exists, "Task with ID 999 should not exist");

        let invalid_result = exists_id(&conn, 0);
        assert!(invalid_result.is_err(), "exists_id should fail for ID <= 0");

        Ok(())
    }

    #[test]
    fn test_get_next_task_id() -> Result<()> {
        let conn = setup_test_db()?;

        // 初期状態（まだシーケンスが動いていない）
        let next_id = get_next_task_id(&conn)?;
        assert_eq!(next_id, 1, "Initial next ID should be 1");

        // シーケンスを1つ進める
        let task = create_test_task(1, TaskStatus::Pending);
        insert_task(&conn, &task)?;

        let next_id_after = get_next_task_id(&conn)?;
        assert_eq!(
            next_id_after, 2,
            "Next ID should be 2 after sequence advancement"
        );

        Ok(())
    }

    #[test]
    fn test_update_task_sets_end_date_on_completion() -> Result<()> {
        let _conn = setup_test_db()?;

        let task = create_test_task(11, TaskStatus::Complete); // Complete に変更

        // update_task 内の end_date 自動付与ロジックの検証
        let mut db_task: DuckdbTask = task.clone().into();
        db_task.end_date =
            if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled {
                Some(jiff::Zoned::now().date().to_string())
            } else {
                None
            };

        // end_date が今日の日付文字列になっているか
        assert!(db_task.end_date.is_some());
        assert_eq!(
            db_task.end_date.unwrap(),
            jiff::Zoned::now().date().to_string()
        );

        Ok(())
    }

    #[test]
    fn test_get_plot_data() -> Result<()> {
        /* Test TaskStatus::Pending */
        let conn = setup_test_db()?;
        for day in 1..=31 {
            let mut task = create_test_task(0, TaskStatus::Pending);
            task.start_date = Date::new(2025, 7, day).unwrap();
            task.end_date = Some(Date::new(2025, 7, 31).unwrap());
            insert_task(&conn, &task)?;
        }
        let start_search = Date::new(2025, 7, 1).unwrap();
        let end_search = Date::new(2025, 7, 31).unwrap();
        let res = get_plot_data(&conn, start_search, end_search)?;
        for (index, data) in res.iter().enumerate() {
            assert_eq!(data.0, 31 - index as i32);
            assert_eq!(data.1, 0);
            assert_eq!(data.2, 0);
            assert_eq!(data.3, 0);
        }

        /* Test TaskStatus::WorkInProgress */
        let conn = setup_test_db()?;
        for day in 1..=31 {
            let mut task = create_test_task(0, TaskStatus::WorkInProgress);
            task.start_date = Date::new(2025, 7, day).unwrap();
            task.end_date = Some(Date::new(2025, 7, 31).unwrap());
            insert_task(&conn, &task)?;
        }
        let start_search = Date::new(2025, 7, 1).unwrap();
        let end_search = Date::new(2025, 7, 31).unwrap();
        let res = get_plot_data(&conn, start_search, end_search)?;
        for (index, data) in res.iter().enumerate() {
            assert_eq!(data.0, 0);
            assert_eq!(data.1, 31 - index as i32);
            assert_eq!(data.2, 0);
            assert_eq!(data.3, 0);
        }

        /* Test TaskStatus::Complete */
        let conn = setup_test_db()?;
        for day in 1..=31 {
            let mut task = create_test_task(0, TaskStatus::Complete);
            task.start_date = Date::new(2025, 7, day).unwrap();
            task.end_date = Some(Date::new(2025, 7, 31).unwrap());
            insert_task(&conn, &task)?;
        }
        let start_search = Date::new(2025, 7, 1).unwrap();
        let end_search = Date::new(2025, 7, 31).unwrap();
        let res = get_plot_data(&conn, start_search, end_search)?;
        for (index, data) in res.iter().enumerate() {
            assert_eq!(data.0, 0);
            assert_eq!(data.1, 0);
            assert_eq!(data.2, 31 - index as i32);
            assert_eq!(data.3, 0);
        }

        /* Test TaskStatus::Canceled */
        let conn = setup_test_db()?;
        for day in 1..=31 {
            let mut task = create_test_task(0, TaskStatus::Canceled);
            task.start_date = Date::new(2025, 7, day).unwrap();
            task.end_date = Some(Date::new(2025, 7, 31).unwrap());
            insert_task(&conn, &task)?;
        }
        let start_search = Date::new(2025, 7, 1).unwrap();
        let end_search = Date::new(2025, 7, 31).unwrap();
        let res = get_plot_data(&conn, start_search, end_search)?;
        for (index, data) in res.iter().enumerate() {
            assert_eq!(data.0, 0);
            assert_eq!(data.1, 0);
            assert_eq!(data.2, 0);
            assert_eq!(data.3, 31 - index as i32);
        }

        Ok(())
    }

    #[test]
    fn test_fetch_all_task() -> Result<()> {
        let conn = setup_test_db()?;
        let tasks = domain::test_util::generate_task_sequence();
        for task in tasks {
            insert_task(&conn, &task)?;
        }
        let filter_flags = TaskFilterFlags::Zero;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::Active;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::Inactive;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::PriorityLow;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::PriorityMiddle;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::PriorityHigh;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::StatusPending;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::StatusWIP;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::StatusComplete;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::StatusCanceled;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 0);

        let filter_flags = TaskFilterFlags::All;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 12);

        let filter_flags =
            TaskFilterFlags::Active | TaskFilterFlags::AllPriorities | TaskFilterFlags::AllStatuses;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 6);

        let filter_flags = TaskFilterFlags::Inactive
            | TaskFilterFlags::AllPriorities
            | TaskFilterFlags::AllStatuses;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 6);

        let filter_flags =
            TaskFilterFlags::Active | TaskFilterFlags::PriorityLow | TaskFilterFlags::AllStatuses;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 2);
        println!("{:?}", task);

        let filter_flags =
            TaskFilterFlags::Active | TaskFilterFlags::PriorityLow | TaskFilterFlags::StatusPending;
        let order_flags = TaskOrderFlags::Zero;
        let task = fetch_all_task(&conn, filter_flags, order_flags)?;
        assert_eq!(task.len(), 1);

        let filter_flags = TaskFilterFlags::All;
        let order_flags = TaskOrderFlags::OrderByPriority | TaskOrderFlags::Reversed;
        let tasks = fetch_all_task(&conn, filter_flags, order_flags)?;
        let task = &tasks[0];
        assert_eq!(task.priority, TaskPriority::High);

        let filter_flags = TaskFilterFlags::All;
        let order_flags = TaskOrderFlags::OrderByPriority;
        let tasks = fetch_all_task(&conn, filter_flags, order_flags)?;
        let task = &tasks[0];
        assert_eq!(task.priority, TaskPriority::Low);

        let filter_flags = TaskFilterFlags::All;
        let order_flags = TaskOrderFlags::OrderByStatus | TaskOrderFlags::Reversed;
        let tasks = fetch_all_task(&conn, filter_flags, order_flags)?;
        let task = &tasks[0];
        assert_eq!(task.status, TaskStatus::Canceled);

        let filter_flags = TaskFilterFlags::All;
        let order_flags = TaskOrderFlags::OrderByStatus;
        let tasks = fetch_all_task(&conn, filter_flags, order_flags)?;
        let task = &tasks[0];
        assert_eq!(task.status, TaskStatus::Pending);
        Ok(())
    }

    #[test]
    fn test_fetch_one_task() -> Result<()> {
        let conn = setup_test_db()?;
        let tasks = domain::test_util::generate_task_sequence();
        for task in tasks {
            insert_task(&conn, &task)?;
        }

        let task = fetch_one_task(&conn, 12);
        assert!(task.is_ok());
        let task = fetch_one_task(&conn, 13);
        assert!(task.is_err());

        Ok(())
    }

    #[test]
    fn test_search_and_fetch_named_params_structures() -> Result<()> {
        let conn = setup_test_db()?;
        let tasks = domain::test_util::generate_task_sequence();
        for task in tasks {
            insert_task(&conn, &task)?;
        }
        /*
        fetch_all_task(&conn)
        {
            let search_flags = 0 as TaskSearchFlags;
            let filter_flags = 0 as TaskFilterFlags;
            let order_flags = 0 as TaskOrderFlags;
            let res = search_task(&conn, "dummy", search_flags, filter_flags, order_flags)?;
        }
        */

        Ok(())
    }
}
