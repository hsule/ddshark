use crate::{
    state::{Abnormality, State},
    utils::GUIDExt,
};
use ratatui::{
    backend::Backend,
    layout::Constraint,
    prelude::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};
use rustdds::GUID;

pub(crate) struct TabAbnormality {
    table_state: TableState,
    num_entries: usize,
}
impl TabAbnormality {
    pub(crate) fn new() -> Self {
        Self {
            table_state: TableState::default(),
            num_entries: 0,
        }
    }

    pub(crate) fn render<B>(&mut self, state: &State, frame: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        const TITLE_WHEN: &str = "when";
        const TITLE_WRITER_ID: &str = "writer";
        const TITLE_READER_ID: &str = "reader";
        const TITLE_TOPIC_NAME: &str = "topic";
        const TITLE_DESC: &str = "desc";

        let mut abnormalities: Vec<_> = state.abnormalities.iter().collect();
        abnormalities.sort_unstable_by(|lhs, rhs| lhs.when.cmp(&rhs.when).reverse());

        let header = vec![
            TITLE_WHEN,
            TITLE_WRITER_ID,
            TITLE_READER_ID,
            TITLE_TOPIC_NAME,
            TITLE_DESC,
        ];
        let rows: Vec<_> = abnormalities
            .into_iter()
            .map(|report| {
                let Abnormality {
                    when,
                    writer_id,
                    reader_id,
                    ref topic_name,
                    ref desc,
                } = *report;
                let guid_to_string = |guid: Option<GUID>| match guid {
                    Some(guid) => format!("{}", guid.display()),
                    None => "<none>".to_string(),
                };

                let when = when.to_rfc3339();
                let reader_id = guid_to_string(reader_id);
                let writer_id = guid_to_string(writer_id);
                let topic_name = topic_name
                    .to_owned()
                    .unwrap_or_else(|| "<none>".to_string());
                let desc = desc.clone();

                vec![when, writer_id, reader_id, topic_name, desc]
            })
            .collect();

        let widths: Vec<_> = header
            .iter()
            .enumerate()
            .map(|(idx, title)| {
                let max_len = rows
                    .iter()
                    .map(|row| row[idx].len())
                    .max()
                    .unwrap_or(0)
                    .max(title.len());
                Constraint::Max(max_len as u16)
            })
            .collect();

        let header = Row::new(header);
        let rows: Vec<_> = rows.into_iter().map(Row::new).collect();

        // Save the # of entires
        self.num_entries = rows.len();

        let table_block = Block::default()
            .title("Abnormalities")
            .borders(Borders::ALL);
        let table = Table::new(rows)
            .style(Style::default().fg(Color::White))
            .header(header)
            .block(table_block)
            .widths(&widths)
            .column_spacing(1)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">");

        frame.render_stateful_widget(table, rect, &mut self.table_state);
    }

    pub(crate) fn previous_item(&mut self) {
        if self.num_entries > 0 {
            let new_idx = match self.table_state.selected() {
                Some(idx) => idx.saturating_sub(1),
                None => 0,
            };
            self.table_state.select(Some(new_idx));
        }
    }

    pub(crate) fn next_item(&mut self) {
        if let Some(last_idx) = self.num_entries.checked_sub(1) {
            let new_idx = match self.table_state.selected() {
                Some(idx) => idx.saturating_add(1).min(last_idx),
                None => 0,
            };
            self.table_state.select(Some(new_idx));
        }
    }

    pub(crate) fn previous_page(&mut self) {
        if self.num_entries > 0 {
            let new_idx = match self.table_state.selected() {
                Some(idx) => idx.saturating_sub(30),
                None => 0,
            };
            self.table_state.select(Some(new_idx));
        }
    }

    pub(crate) fn next_page(&mut self) {
        if let Some(last_idx) = self.num_entries.checked_sub(1) {
            let new_idx = match self.table_state.selected() {
                Some(idx) => idx.saturating_add(30).min(last_idx),
                None => 0,
            };
            self.table_state.select(Some(new_idx));
        }
    }

    pub(crate) fn first_item(&mut self) {
        if self.num_entries > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub(crate) fn last_item(&mut self) {
        if let Some(idx) = self.num_entries.checked_sub(1) {
            self.table_state.select(Some(idx));
        }
    }
}
