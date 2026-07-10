use crate::duckdb_work_time::DuckdbWorkTime;
use anyhow::Result;
use domain::WorkTime;
use duckdb::Connection;
use jiff::civil::Date;
use tracing::info;

pub fn next_work_time_id(conn: &Connection) -> Result<i32> {
    let query =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'seq_work_time_id';";
    let last_value: i64 = conn.query_row(query, [], |row| row.get(0)).unwrap_or(0);
    let next_id = (last_value + 1) as i32;
    info!("Next work time id is {}.", next_id);

    Ok(next_id)
}

pub fn find_work_time(conn: &Connection, task_id: i32, date: &Date) -> Result<Option<WorkTime>> {
    info!("Find work_time for task_id: {}, date: {}", task_id, date);
    const FIND_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/find_work_time.sql");
    let mut stmt = conn.prepare(FIND_WORK_TIME_SQL)?;
    let params = duckdb::named_params! {
        "task_id": task_id,
        "date": date.to_string(),
    };
    let mut rows = stmt.query(params)?;
    if let Some(row) = rows.next()? {
        let duckdb_work_time = DuckdbWorkTime::try_from(row)?;
        let work_time = WorkTime::try_from(duckdb_work_time)?;
        Ok(Some(work_time))
    } else {
        Ok(None)
    }
}

pub fn list_work_time_for_task(conn: &Connection, task_id: i32) -> Result<Vec<WorkTime>> {
    info!("List work_time for task_id: {}", task_id);

    const LIST_WORK_TIME_FOR_TASK_SQL: &str =
        include_str!("../assets/work_time_sql/list_work_time_by_task.sql");
    let mut stmt = conn.prepare(LIST_WORK_TIME_FOR_TASK_SQL)?;
    let params = duckdb::named_params! {
        "task_id": task_id,
    };
    let work_time_iter = stmt.query_map(params, |row| DuckdbWorkTime::try_from(row))?;

    let mut result = Vec::new();
    for item in work_time_iter {
        let duckdb_item = item?;
        let work_time = WorkTime::try_from(duckdb_item)?;
        result.push(work_time);
    }

    Ok(result)
}

pub fn insert_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    info!("Inserting work time: {:?}", work_time);
    const INSERT_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/insert_work_time.sql");
    let duckdb_work_time: DuckdbWorkTime = work_time.clone().into();
    let mut params = duckdb_work_time.to_named_params();
    let _ = params.remove("id");
    let mut stmt = conn.prepare(INSERT_WORK_TIME_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn update_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    info!("Update work time: {:?}", work_time);
    const UPDATE_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/update_work_time.sql");
    let duckdb_work_time: DuckdbWorkTime = work_time.clone().into();
    let params = duckdb_work_time.to_named_params();
    let mut stmt = conn.prepare(UPDATE_WORK_TIME_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn upsert_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    let exists = exists_work_time_id(conn, work_time.id)?;
    if exists {
        update_work_time(conn, work_time)
    } else {
        insert_work_time(conn, work_time)
    }
}

fn exists_work_time_id(conn: &Connection, id: i32) -> Result<bool> {
    info!("Check id {} exists in work_time table", id);
    const EXISTS_WORK_TIME_ID_SQL: &str =
        include_str!("../assets/work_time_sql/exists_work_time_id.sql");

    let params = duckdb::named_params! {
        "id": id,
    };

    let exists: bool = conn.query_row(EXISTS_WORK_TIME_ID_SQL, params, |row| row.get(0))?;

    Ok(exists)
}

pub fn get_total_work_time_by_task(conn: &Connection, task_id: i32) -> Result<f32> {
    info!("Get total work time for: {}", task_id);
    const GET_TOTAL_WORK_TIME_BY_TASK_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_by_task.sql");
    let params = duckdb::named_params! {"task_id": task_id};
    let total_time: f32 = conn.query_row(
        GET_TOTAL_WORK_TIME_BY_TASK_SQL,
        params,
        |row| row.get(0), // 最初のカラム（SUMの結果）を取得
    )?;

    Ok(total_time)
}

pub fn get_total_work_time_by_date(conn: &Connection, date: &Date) -> Result<f32> {
    info!("Get total work time for: {}", date);
    const GET_TOTAL_WORK_TIME_BY_DATE_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_by_date.sql");
    let date_str = date.to_string();
    let params = duckdb::named_params! {"date": date_str};
    let total_time: f32 = conn.query_row(
        GET_TOTAL_WORK_TIME_BY_DATE_SQL,
        params,
        |row| row.get(0), // 最初のカラム（SUMの結果）を取得
    )?;

    Ok(total_time)
}

pub fn get_total_work_time_history(
    conn: &Connection,
    start_date: &Date,
    end_date: &Date,
) -> Result<Vec<(Date, f32)>> {
    info!(
        "Get total work time history from {} to {}",
        start_date, end_date
    );
    const GET_TOTAL_WORK_TIME_HISTORY_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_history.sql");

    let start_date_str = start_date.to_string();
    let end_date_str = end_date.to_string();

    let params = duckdb::named_params! {
        "start_date": start_date_str,
        "end_date": end_date_str
    };

    let mut stmt = conn.prepare(GET_TOTAL_WORK_TIME_HISTORY_SQL)?;
    let mut rows = stmt.query(params)?;

    let mut history = Vec::new();
    while let Some(row) = rows.next()? {
        let date_str: String = row.get(0)?;
        let date = date_str.parse::<Date>()?;
        let time_spent: f32 = row.get(1)?;
        history.push((date, time_spent));
    }

    Ok(history)
}

pub fn get_total_work_time_ratio(
    conn: &Connection,
    start_date: &Date,
    end_date: &Date,
) -> Result<Vec<(i32, f32)>> {
    info!(
        "Get total work time ratio from {} to {}",
        start_date, end_date
    );
    const GET_TOTAL_WORK_TIME_RATIO_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_ratio.sql");

    let start_date_str = start_date.to_string();
    let end_date_str = end_date.to_string();

    let params = duckdb::named_params! {
        "start_date": start_date_str,
        "end_date": end_date_str
    };

    let mut stmt = conn.prepare(GET_TOTAL_WORK_TIME_RATIO_SQL)?;
    let mut rows = stmt.query(params)?;

    let mut ratios = Vec::new();
    while let Some(row) = rows.next()? {
        let task_id: i32 = row.get(0)?;
        let ratio: f32 = row.get(1)?;

        ratios.push((task_id, ratio));
    }

    Ok(ratios)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DuckdbPath, connect};
    use anyhow::Result;

    use duckdb::Connection;
    use jiff::civil::Date;

    // --- テスト用のヘルパー関数 ---
    // インメモリのDuckDBを初期化し、必要なテーブルとシーケンスを作成します
    fn setup_test_db() -> Result<Connection> {
        let path = DuckdbPath::InMemory;
        connect(&path)
    }

    // テスト用のWorkTimeインスタンスを生成するヘルパー（フィールドは実際の定義に合わせる）
    fn create_dummy_work_time(id: i32, task_id: i32, date: &Date, time_spent: f32) -> WorkTime {
        WorkTime {
            id,
            task_id,
            date: *date,
            time_spent,
        }
    }

    // --- 各関数のテストケース ---

    #[test]
    fn test_next_work_time_id() -> Result<()> {
        let conn = setup_test_db()?;

        // 初期状態（シーケンスがまだ呼ばれていない時）はlast_valueがないためデフォルトの 1 が返る
        let id1 = next_work_time_id(&conn).unwrap();
        assert_eq!(id1, 1);

        // シーケンスを進める
        conn.execute("SELECT nextval('seq_work_time_id');", [])
            .unwrap();

        // last_valueが1になるため、1 + 1 = 2 が返る
        let id2 = next_work_time_id(&conn).unwrap();
        assert_eq!(id2, 2);
        Ok(())
    }

    #[test]
    fn test_insert_and_find_work_time() -> Result<()> {
        let conn = setup_test_db()?;
        let date = Date::new(2026, 7, 9).unwrap();
        let work_time = create_dummy_work_time(1, 100, &date, 2.5);

        // データがない状態での検索は None
        let found_none = find_work_time(&conn, 100, &date).unwrap();
        assert!(found_none.is_none());

        // インサートの実行
        insert_work_time(&conn, &work_time).unwrap();

        // インサートしたデータの検索
        let found_some = find_work_time(&conn, 100, &date).unwrap();
        assert!(found_some.is_some());
        let fetched = found_some.unwrap();
        assert_eq!(fetched.id, 1);
        assert_eq!(fetched.task_id, 100);
        assert_eq!(fetched.time_spent, 2.5);
        Ok(())
    }

    #[test]
    fn test_list_work_time_for_task() -> Result<()> {
        let conn = setup_test_db()?;
        let date1 = Date::new(2026, 7, 9).unwrap();
        let date2 = Date::new(2026, 7, 10).unwrap();

        let wt1 = create_dummy_work_time(1, 100, &date1, 1.5);
        let wt2 = create_dummy_work_time(2, 100, &date2, 3.0);
        let wt3 = create_dummy_work_time(3, 200, &date1, 4.0); // 別のタスク

        insert_work_time(&conn, &wt1).unwrap();
        insert_work_time(&conn, &wt2).unwrap();
        insert_work_time(&conn, &wt3).unwrap();

        // タスク100のリストを取得（2件のはず）
        let list = list_work_time_for_task(&conn, 100).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|w| w.id == 1));
        assert!(list.iter().any(|w| w.id == 2));
        Ok(())
    }

    #[test]
    fn test_upsert_and_exists_work_time() -> Result<()> {
        let conn = setup_test_db()?;
        let date = Date::new(2026, 7, 9).unwrap();
        let work_time = create_dummy_work_time(1, 100, &date, 2.0);

        // 初期状態で存在確認は false
        assert!(!exists_work_time_id(&conn, 1).unwrap());

        // 1回目のupsert (insertが走る)
        upsert_work_time(&conn, &work_time).unwrap();
        assert!(exists_work_time_id(&conn, 1).unwrap());

        // 値を変更して2回目のupsert (updateが走る)
        let updated_work_time = create_dummy_work_time(1, 100, &date, 5.5);
        upsert_work_time(&conn, &updated_work_time).unwrap();

        // 変更が反映されているか確認
        let found = find_work_time(&conn, 100, &date).unwrap().unwrap();
        assert_eq!(found.time_spent, 5.5);
        Ok(())
    }

    #[test]
    fn test_get_total_work_time_by_task_and_date() -> Result<()> {
        let conn = setup_test_db()?;
        let date1 = Date::new(2026, 7, 9).unwrap();
        let date2 = Date::new(2026, 7, 10).unwrap();

        insert_work_time(&conn, &create_dummy_work_time(1, 100, &date1, 1.5)).unwrap();
        insert_work_time(&conn, &create_dummy_work_time(2, 100, &date2, 2.5)).unwrap();
        insert_work_time(&conn, &create_dummy_work_time(3, 200, &date1, 3.0)).unwrap();

        // タスクごとの合計時間
        let total_task_100 = get_total_work_time_by_task(&conn, 100).unwrap();
        assert_eq!(total_task_100, 4.0); // 1.5 + 2.5

        // 日付ごとの合計時間
        let total_date_1 = get_total_work_time_by_date(&conn, &date1).unwrap();
        assert_eq!(total_date_1, 4.5); // 1.5 + 3.0
        Ok(())
    }

    #[test]
    fn test_get_total_work_time_history_and_ratio() -> Result<()> {
        let conn = setup_test_db()?;
        let start_date = Date::new(2026, 7, 8).unwrap();
        let date1 = Date::new(2026, 7, 9).unwrap();
        let date2 = Date::new(2026, 7, 10).unwrap();
        let end_date = Date::new(2026, 7, 11).unwrap();

        insert_work_time(&conn, &create_dummy_work_time(1, 100, &date1, 2.0)).unwrap();
        insert_work_time(&conn, &create_dummy_work_time(2, 200, &date2, 3.0)).unwrap();

        // 履歴のテスト
        let history = get_total_work_time_history(&conn, &start_date, &end_date).unwrap();
        // SQLの実装（GROUP BYなど）によりますが、データが存在する日付のペアが含まれているか検証
        assert!(history.iter().any(|(d, t)| *d == date1 && *t == 2.0));
        assert!(history.iter().any(|(d, t)| *d == date2 && *t == 3.0));

        // 比率/割合のテスト
        let ratios = get_total_work_time_ratio(&conn, &start_date, &end_date).unwrap();
        assert!(
            ratios
                .iter()
                .any(|(task_id, ratio)| *task_id == 100 && *ratio > 0.0)
        );
        assert!(
            ratios
                .iter()
                .any(|(task_id, ratio)| *task_id == 200 && *ratio > 0.0)
        );
        Ok(())
    }
}
