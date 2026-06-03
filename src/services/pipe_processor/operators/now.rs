use chrono::{DateTime, Local};

use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Now;

impl Now {
    pub fn new() -> Self {
        Self {}
    }

    fn format_time(&self, date_time: &DateTime<Local>) -> String {
        date_time.format("%H:%M").to_string()
    }

    fn format_date(&self, date_time: &DateTime<Local>) -> String {
        date_time.format("%d.%m.%Y").to_string()
    }

    fn format_date_time(&self, date_time: &DateTime<Local>) -> String {
        let time = self.format_time(date_time);
        let date = self.format_date(date_time);

        format!("{date}, {time}")
    }
}

impl PipeOperator for Now {
    fn handle(&self, type_or_caption: &str) -> anyhow::Result<String> {
        let now = Local::now();
        match type_or_caption {
            _ if type_or_caption.to_lowercase() == "date" => Ok(self.format_date(&now)),
            _ if type_or_caption.to_lowercase() == "time" => Ok(self.format_time(&now)),
            _ => Ok(self.format_date_time(&now)),
        }
    }
}
