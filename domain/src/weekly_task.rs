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

#[allow(clippy::trivially_copy_pass_by_ref)]
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

fn deserialize_weekday<'de, D>(deserializer: D) -> std::result::Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // マッチしない場合は、serde::de::Error::custom を使ってエラーを返します
    match s.as_str() {
        "Monday" | "monday" => Ok(Weekday::Monday),
        "Tuesday" | "tuesday" => Ok(Weekday::Tuesday),
        "Wednesday" | "wednesday" => Ok(Weekday::Wednesday),
        "Thursday" | "thursday" => Ok(Weekday::Thursday),
        "Friday" | "friday" => Ok(Weekday::Friday),
        "Saturday" | "saturday" => Ok(Weekday::Saturday),
        "Sunday" | "sunday" => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom(format!(
            "invalid weekday string: '{s}'. Expected standard full weekday name (e.g., 'Monday')"
        ))),
    }
}

impl WeeklyTask {
    #[must_use]
    pub fn is_saveable(&self) -> bool {
        !self.project.trim().is_empty() && !self.title.trim().is_empty()
    }

    pub fn create_task(&self, today: &Date) -> Result<Task> {
        if !self.is_saveable() {
            bail!("Invalid weekly task");
        }

        let task_start_date = if today.weekday() == self.start_day {
            *today
        } else {
            today.nth_weekday(1, self.start_day)?
        };

        let task_due_date = task_start_date.yesterday()?.nth_weekday(1, self.due_day)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    fn valid_weekly_task() -> WeeklyTask {
        WeeklyTask {
            id: 200,
            active: true,
            project: "Reporting".to_string(),
            title: "Weekly Status".to_string(),
            detail: "Submit weekly report".to_string(),
            priority: TaskPriority::Medium,
            start_day: Weekday::Monday,
            due_day: Weekday::Friday,
        }
    }

    // =========================================================================
    // 基本挙動 & カレンダー計算のテスト（依存なし）
    // =========================================================================
    #[test]
    fn test_default_impl() {
        let default_task = WeeklyTask::default();
        assert_eq!(default_task.id, 0);
        assert!(default_task.active);
        assert_eq!(default_task.start_day, Weekday::Monday);
        assert_eq!(default_task.due_day, Weekday::Friday);
    }

    #[test]
    fn test_is_saveable_success() {
        let task = valid_weekly_task();
        assert!(task.is_saveable());
    }

    #[test]
    fn test_is_saveable_failures() {
        let mut task = valid_weekly_task();
        task.project = "   ".to_string();
        assert!(!task.is_saveable());
    }

    #[test]
    fn test_create_task_date_calculation() {
        let weekly_task = valid_weekly_task();
        let today = Date::new(2026, 7, 8).unwrap(); // 水曜日

        let created_task = weekly_task.create_task(&today).unwrap();

        assert_eq!(created_task.start_date, Date::new(2026, 7, 13).unwrap()); // 直近の月曜
        assert_eq!(created_task.due_date, Date::new(2026, 7, 17).unwrap()); // 直近の金曜
        assert_eq!(created_task.title, "Weekly Status for July Week 2");
        assert_eq!(created_task.entry_date, today);
    }
}
