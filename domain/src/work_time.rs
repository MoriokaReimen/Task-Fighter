use jiff::civil::Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkTime {
    pub id: i32,
    pub task_id: i32,
    pub date: Date,
    pub time_spent: f32,
}

impl WorkTime {
    pub fn is_saveable(&self) -> bool {
        self.task_id > 0
    }

    pub fn accumulate_time(&mut self, seconds: i64) {
        let hours = (seconds as f32) / 3600.0;
        self.time_spent += hours;
        self.time_spent = (self.time_spent * 10.0).round() / 10.0;
    }
}

impl Default for WorkTime {
    fn default() -> Self {
        Self {
            id: 0,
            task_id: 0,
            date: Date::new(1970, 1, 1).unwrap(),
            time_spent: 0f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    // 共通で使える有効なWorkTimeのヘルパー
    fn valid_work_time() -> WorkTime {
        WorkTime {
            id: 1,
            task_id: 42,
            date: Date::new(2026, 7, 7).unwrap(),
            time_spent: 0.0,
        }
    }

    // =========================================================================
    // Default 実装のテスト
    // =========================================================================
    #[test]
    fn test_default_impl() {
        let default_work = WorkTime::default();
        assert_eq!(default_work.id, 0);
        assert_eq!(default_work.task_id, 0);
        assert_eq!(default_work.date, Date::new(1970, 1, 1).unwrap());
        assert!((default_work.time_spent - 0.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // バリデーション (is_saveable) のテスト
    // =========================================================================
    #[test]
    fn test_is_saveable() {
        let mut work = valid_work_time();

        // 正常系: task_id が 0 より大きい
        assert!(work.is_saveable());

        // 異常系: task_id が 0
        work.task_id = 0;
        assert!(!work.is_saveable());

        // 異常系: task_id が 負の数
        work.task_id = -1;
        assert!(!work.is_saveable());
    }

    // =========================================================================
    // 時間累積 (accumulate_time) のテスト
    // =========================================================================
    #[test]
    fn test_accumulate_time_basic() {
        let mut work = valid_work_time();

        // 1時間 (3600秒) を追加
        work.accumulate_time(3600);
        assert!((work.time_spent - 1.0).abs() < f32::EPSILON);

        // さらに30分 (1800秒) を追加 -> 1.5時間
        work.accumulate_time(1800);
        assert!((work.time_spent - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_accumulate_time_rounding() {
        let mut work = valid_work_time();

        // 20分 (1200秒) = 0.3333... 時間 -> 四捨五入で 0.3 になるか
        work.accumulate_time(1200);
        assert!((work.time_spent - 0.3).abs() < f32::EPSILON);

        // さらに45分 (2700秒) 追加 = +0.75時間
        // 0.3 + 0.75 = 1.05 -> 小数点第1位に四捨五入されて 1.1 になるか
        work.accumulate_time(2700);
        assert!((work.time_spent - 1.1).abs() < f32::EPSILON);

        // 1秒単位の微小時間 -> 0.00027... 時間 (1.1のまま変化しない)
        work.accumulate_time(1);
        assert!((work.time_spent - 1.1).abs() < f32::EPSILON);
    }
}
