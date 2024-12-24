use chrono::{DateTime, Datelike, Local, NaiveDate};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub current_date: DateTime<Local>,
    pub selected_date: DateTime<Local>,
}

impl Calendar {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            current_date: now,
            selected_date: now,
        }
    }

    pub fn next_month(&mut self) {
        let naive_date = self.current_date.naive_local().date();
        let next_month = if naive_date.month() == 12 {
            NaiveDate::from_ymd_opt(
                naive_date.year() + 1,
                1,
                1
            ).unwrap()
        } else {
            NaiveDate::from_ymd_opt(
                naive_date.year(),
                naive_date.month() + 1,
                1
            ).unwrap()
        };
        self.current_date = DateTime::from_naive_utc_and_offset(
            next_month.and_hms_opt(0, 0, 0).unwrap(),
            *self.current_date.offset()
        );
    }

    pub fn prev_month(&mut self) {
        let naive_date = self.current_date.naive_local().date();
        let prev_month = if naive_date.month() == 1 {
            NaiveDate::from_ymd_opt(
                naive_date.year() - 1,
                12,
                1
            ).unwrap()
        } else {
            NaiveDate::from_ymd_opt(
                naive_date.year(),
                naive_date.month() - 1,
                1
            ).unwrap()
        };
        self.current_date = DateTime::from_naive_utc_and_offset(
            prev_month.and_hms_opt(0, 0, 0).unwrap(),
            *self.current_date.offset()
        );
    }

    pub fn get_month_grid(&self) -> Vec<Vec<Option<u32>>> {
        let naive_date = self.current_date.naive_local().date();
        let first_day = NaiveDate::from_ymd_opt(
            naive_date.year(),
            naive_date.month(),
            1
        ).unwrap();
        
        let days_in_month = if naive_date.month() == 12 {
            NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
        }.unwrap()
        .signed_duration_since(first_day)
        .num_days() as u32;

        let first_weekday = first_day.weekday().num_days_from_sunday();
        let mut grid = vec![vec![None; 7]; 6];
        let mut current_day = 1;

        for week in 0..6 {
            for day in 0..7 {
                if week == 0 && day < first_weekday {
                    continue;
                }
                if current_day <= days_in_month {
                    grid[week as usize][day as usize] = Some(current_day);
                    current_day += 1;
                }
            }
        }
        grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_creation() {
        let calendar = Calendar::new();
        assert!(calendar.current_date <= Local::now());
    }

    #[test]
    fn test_month_navigation() {
        let mut calendar = Calendar::new();
        let initial_month = calendar.current_date.month();
        
        calendar.next_month();
        assert_eq!(
            calendar.current_date.month(),
            if initial_month == 12 { 1 } else { initial_month + 1 }
        );

        calendar.prev_month();
        assert_eq!(calendar.current_date.month(), initial_month);
    }
}