//! `/admin`: the admin hub (M7). One table of admin areas with a plain-words
//! status each, including what is still waiting for the owner's approval.
//! Non-admins get the 404 page (`SsrMode::Async`).

use leptos::prelude::*;

use crate::components::ui::{
    PageHeader, PageShell, PageWidth, RowLink, Skeleton, SkeletonGroup, Table, Td, Th, Tr,
};
use crate::error_template::NotFound;
use crate::models::community::{month_label, AdminOverview};

#[server]
pub async fn admin_hub_overview() -> Result<(String, AdminOverview), ServerFnError> {
    use crate::server::{authz, community::store, db};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let mut o = store::admin_overview(&db)
        .await
        .map_err(authz::server_err)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    o.invites_open = db::list_invites(&db)
        .await
        .map_err(authz::server_err)?
        .iter()
        .filter(|i| i.status(&now) == "open")
        .count() as u32;
    Ok((admin.username, o))
}

fn plural(n: u32, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// (area, link, status) rows of the hub.
pub fn hub_rows(o: &AdminOverview) -> Vec<(&'static str, &'static str, String)> {
    let bugs = if o.bugs_new + o.bugs_triaged == 0 {
        "Nothing waiting".to_string()
    } else {
        format!("{} new, {} triaged", o.bugs_new, o.bugs_triaged)
    };
    let surveys = format!(
        "{} waiting for approval, {} live",
        o.surveys_waiting, o.surveys_live
    );
    let proposals = format!(
        "{}, {}, {} not published",
        plural(o.proposals_open, "open idea", "open ideas"),
        plural(o.votes, "vote", "votes"),
        o.proposals_unpublished
    );
    let ledger = match &o.ledger_last_published {
        Some(m) => format!("Last published: {}", month_label(m)),
        None => "Nothing published yet".to_string(),
    } + &if o.ledger_drafts > 0 {
        format!(", {}", plural(o.ledger_drafts, "draft", "drafts"))
    } else {
        String::new()
    };
    let texts = if o.texts_waiting == 0 {
        format!("All {} approved", o.texts_total)
    } else {
        format!(
            "{} of {} waiting for approval",
            o.texts_waiting, o.texts_total
        )
    };
    let invites = if o.invites_open == 0 {
        "No open codes".to_string()
    } else {
        plural(o.invites_open, "open code", "open codes")
    };
    vec![
        ("Bug reports", "/admin/bugs", bugs),
        ("Surveys", "/admin/surveys", surveys),
        ("What next", "/admin/proposals", proposals),
        ("Transparency ledger", "/admin/ledger", ledger),
        ("Public texts", "/admin/texts", texts),
        ("Invites", "/admin/invites", invites),
        (
            "Password resets",
            "/admin/resets",
            "One-time codes for forgotten passwords".to_string(),
        ),
        (
            "Champion traits",
            "/admin/traits",
            "Draft Graph trait table".to_string(),
        ),
        (
            "Design system",
            "/design",
            "Every component and token".to_string(),
        ),
    ]
}

#[component]
pub fn AdminHubPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| admin_hub_overview());
    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((name, o)) => view! {
                    <PageShell width=PageWidth::Standard>
                        <PageHeader
                            title="Admin"
                            eyebrow="Admin"
                            meta=move || view! { <span>{format!("Signed in as {name}. Admins are set with ADMIN_EMAILS on the server.")}</span> }
                        />
                        <Table caption="Admin areas">
                            <thead>
                                <tr>
                                    <Th>"Area"</Th>
                                    <Th>"Status"</Th>
                                </tr>
                            </thead>
                            <tbody>
                                {hub_rows(&o).into_iter().map(|(area, href, status)| view! {
                                    <Tr>
                                        <Td header=true><RowLink href=href>{area}</RowLink></Td>
                                        <Td muted=true>{status}</Td>
                                    </Tr>
                                }).collect_view()}
                            </tbody>
                        </Table>
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hub_lists_every_admin_area() {
        let rows = hub_rows(&AdminOverview::default());
        let links: Vec<&str> = rows.iter().map(|r| r.1).collect();
        for l in [
            "/admin/bugs",
            "/admin/surveys",
            "/admin/proposals",
            "/admin/ledger",
            "/admin/texts",
            "/admin/invites",
            "/admin/resets",
            "/admin/traits",
            "/design",
        ] {
            assert!(links.contains(&l), "{l}");
        }
        assert_eq!(rows[0].2, "Nothing waiting");
        assert_eq!(rows[3].2, "Nothing published yet");
    }
}
