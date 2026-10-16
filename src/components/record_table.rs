//! RecordTable: a sortable "games, won, win rate" table for one kind of
//! group (a strategy tag, a composition tag, a first pick). M141: every
//! table of comparable rows sorts by its columns (report rucyy6k1).

use leptos::prelude::*;

use crate::components::ui::{sort_rows, SortState, SortStatus, SortTh, Table, Td, Tr};

const LABELS: &[(&str, &str)] = &[
    ("name", "name"),
    ("games", "Games"),
    ("won", "Won"),
    ("rate", "Win rate"),
];

fn rate(games: i32, wins: i32) -> i32 {
    if games > 0 {
        (wins as f64 / games as f64 * 100.0).round() as i32
    } else {
        0
    }
}

/// Rows in the table's order (stable; unsorted is the incoming order).
pub fn sort_records(rows: &mut [(String, i32, i32)], sort: SortState) {
    let Some(key) = sort.key else { return };
    sort_rows(rows, sort.dir, |a, b| match key {
        "name" => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        "games" => a.1.cmp(&b.1),
        "won" => a.2.cmp(&b.2),
        "rate" => (a.2 as f64 / a.1.max(1) as f64).total_cmp(&(b.2 as f64 / b.1.max(1) as f64)),
        _ => std::cmp::Ordering::Equal,
    });
}

/// A sortable record table: first column `first` (the group's name), then
/// games, won and win rate. Starts in the order the rows came in.
///
/// A11y: a [`Table`] with [`SortTh`] headers (`aria-sort`, "Sort by ..."
/// buttons) and one [`SortStatus`] announcing the order; the row named by
/// `current` is marked selected (`aria-selected`) and tinted.
#[component]
pub fn RecordTable(
    #[prop(into)] caption: String,
    #[prop(optional)] caption_hidden: bool,
    first: &'static str,
    rows: Vec<(String, i32, i32)>,
    /// The row to mark (the plan's own strategy).
    #[prop(optional, into)]
    current: Option<String>,
) -> impl IntoView {
    let sort = RwSignal::new(SortState::none());
    let rows = StoredValue::new(rows);
    let current = StoredValue::new(current);
    view! {
        <SortStatus sort=sort labels=LABELS />
        <Table caption=caption caption_hidden=caption_hidden>
            <thead>
                <tr>
                    <SortTh label=first key="name" sort=sort />
                    <SortTh label="Games" key="games" sort=sort numeric=true />
                    <SortTh label="Won" key="won" sort=sort numeric=true />
                    <SortTh label="Win rate" key="rate" sort=sort numeric=true />
                </tr>
            </thead>
            <tbody>
                {move || {
                    let mut list = rows.get_value();
                    sort_records(&mut list, sort.get());
                    let cur = current.get_value();
                    list.into_iter().map(|(name, games, wins)| {
                        let selected = cur.as_deref() == Some(name.as_str());
                        view! {
                            <Tr selected=selected>
                                <Td header=true>{name}</Td>
                                <Td numeric=true>{games}</Td>
                                <Td numeric=true>{wins}</Td>
                                <Td numeric=true>{format!("{}%", rate(games, wins))}</Td>
                            </Tr>
                        }
                    }).collect_view()
                }}
            </tbody>
        </Table>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::SortDir;

    #[test]
    fn records_sort_by_rate_and_name() {
        let mut rows = vec![
            ("poke".to_string(), 4, 1),
            ("Dive".to_string(), 2, 2),
            ("engage".to_string(), 4, 3),
        ];
        sort_records(&mut rows, SortState::by("rate", SortDir::Descending));
        assert_eq!(rows[0].0, "Dive");
        assert_eq!(rows[2].0, "poke");
        sort_records(&mut rows, SortState::by("name", SortDir::Ascending));
        let names: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(names, vec!["Dive", "engage", "poke"]);
    }
}
