use crate::task::{Task, TaskPriority};
use anyhow::{Result, bail};
use jiff::civil::{Date, Weekday};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WeeklyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
    #[serde(
        serialize_with = "serialize_weekday",
        deserialize_with = "deserialize_weekday"
    )]
    pub start_day: Weekday,

    #[serde(
        serialize_with = "serialize_weekday",
        deserialize_with = "deserialize_weekday"
    )]
    pub due_day: Weekday,
}

impl Default for WeeklyTask {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: TaskPriority::Low,
            start_day: Weekday::Monday,
            due_day: Weekday::Friday,
        }
    }
}

fn serialize_weekday<S>(weekday: &Weekday, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let name = match weekday {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    };
    serializer.serialize_str(name)
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // s.as_str() で &str に変換することで、文字列リテラルで match できるようになります
    let ret = match s.as_str() {
        "Monday" => Weekday::Monday,
        "Tuesday" => Weekday::Tuesday,
        "Wednesday" => Weekday::Wednesday,
        "Thursday" => Weekday::Thursday,
        "Friday" => Weekday::Friday,
        "Saturday" => Weekday::Saturday,
        "Sunday" => Weekday::Sunday,
        _ => Weekday::Sunday, // マッチしない場合のデフォルト値
    };

    Ok(ret)
}

impl WeeklyTask {
    pub fn is_valid(&self) -> bool {
        !self.project.trim().is_empty()
            && !self.title.trim().is_empty()
            && (self.start_day as u8) <= (self.due_day as u8)
    }

    pub fn create_task(&self, today: &Date) -> Result<Task> {
        if !self.is_valid() {
            bail!("Invalid weekly task");
        }

        let calculation_base_date = today.tomorrow()?;

        let task_start_date = calculation_base_date.nth_weekday(-1, self.start_day)?;
        let task_due_date = calculation_base_date.nth_weekday(1, self.due_day)?;

        let formatted_month = task_start_date.strftime("%B");
        let week_of_month = ((task_start_date.day() - 1) / 7) + 1;

        let title_with_week = format!(
            "{} for {} Week {}",
            self.title, formatted_month, week_of_month
        );

        Ok(Task {
            id: self.id,
            project: self.project.clone(),
            title: title_with_week,
            detail: self.detail.clone(),
            priority: self.priority,
            start_date: task_start_date,
            due_date: task_due_date,
            entry_date: *today,
            ..Task::default()
        })
    }
}
