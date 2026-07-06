use jiff::civil::Date;

pub struct WorkTime {
    pub id: i32,
    pub task_id: i32,
    pub date: Date,
    pub time_spent: f32,
}

impl WorkTime {
    pub fn is_saveable(&self) -> bool {
        self.task_id != 0
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
