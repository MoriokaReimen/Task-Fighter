use anyhow::Result;

fn main() -> Result<()> {
    let path = driver::DuckdbPath::InDirectory("./runtime".into());
    let conn = driver::connect(&path)?;
    let tasks = domain::test_util::generate_random_tasks(1000);
    for task in tasks {
        let _ = driver::insert_task(&conn, &task);
    }

    Ok(())
}
