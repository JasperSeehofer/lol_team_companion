use leptos::prelude::*;
use crate::models::draft::{DraftTreeNode, DraftAction};
use crate::models::champion::Champion;

// Layout constants
const NODE_W: f64 = 150.0;
const NODE_H: f64 = 36.0;
const LEVEL_H: f64 = 100.0;
const H_GAP: f64 = 16.0;
const ICON_SIZE: f64 = 22.0;

/// Positioned node for the layout algorithm
#[derive(Clone, Debug)]
struct LayoutNode {
    id: String,
    label: String,
    is_improvised: bool,
    is_root: bool,
    actions: Vec<DraftAction>,
    children: Vec<LayoutNode>,
    // Computed by layout
    x: f64,
    y: f64,
    width: f64,
}

/// An edge between two nodes, with the champion diff
#[derive(Clone, Debug)]
struct Edge {
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
    /// Champions that appear in child but not parent (the new pick/ban)
    diff_champions: Vec<DiffChampion>,
}

#[derive(Clone, Debug)]
struct DiffChampion {
    name: String,
    is_ban: bool,
}

/// Convert tree nodes into layout nodes (recursive)
fn to_layout_nodes(nodes: &[DraftTreeNode]) -> Vec<LayoutNode> {
    nodes
        .iter()
        .map(|n| LayoutNode {
            id: n.id.clone().unwrap_or_default(),
            label: n.label.clone(),
            is_improvised: n.is_improvised,
            is_root: n.parent_id.is_none(),
            actions: n.actions.clone(),
            children: to_layout_nodes(&n.children),
            x: 0.0,
            y: 0.0,
            width: 0.0,
        })
        .collect()
}

/// Compute subtree width (bottom-up)
fn compute_widths(node: &mut LayoutNode) {
    if node.children.is_empty() {
        node.width = NODE_W;
        return;
    }
    for child in &mut node.children {
        compute_widths(child);
    }
    let children_width: f64 = node
        .children
        .iter()
        .map(|c| c.width)
        .sum::<f64>()
        + H_GAP * (node.children.len() as f64 - 1.0).max(0.0);
    node.width = children_width.max(NODE_W);
}

/// Assign x,y positions (top-down)
fn assign_positions(node: &mut LayoutNode, x: f64, y: f64) {
    node.x = x + node.width / 2.0 - NODE_W / 2.0;
    node.y = y;

    if node.children.is_empty() {
        return;
    }

    let children_total: f64 = node
        .children
        .iter()
        .map(|c| c.width)
        .sum::<f64>()
        + H_GAP * (node.children.len() as f64 - 1.0).max(0.0);

    let start_x = x + (node.width - children_total) / 2.0;
    let child_y = y + LEVEL_H;

    let mut cx = start_x;
    for child in &mut node.children {
        assign_positions(child, cx, child_y);
        cx += child.width + H_GAP;
    }
}

/// Compute champion diff between parent and child actions
fn diff_actions(parent: &[DraftAction], child: &[DraftAction]) -> Vec<DiffChampion> {
    // Find actions in child that don't exist in parent (by order+champion)
    let parent_set: std::collections::HashSet<(i32, &str)> = parent
        .iter()
        .map(|a| (a.order, a.champion.as_str()))
        .collect();

    child
        .iter()
        .filter(|a| !a.champion.is_empty() && !parent_set.contains(&(a.order, a.champion.as_str())))
        .map(|a| DiffChampion {
            name: a.champion.clone(),
            is_ban: a.phase.starts_with("ban"),
        })
        .collect()
}

/// Collect edges from the layout tree (recursive)
fn collect_edges(node: &LayoutNode) -> Vec<Edge> {
    let mut edges = Vec::new();
    let from_x = node.x + NODE_W / 2.0;
    let from_y = node.y + NODE_H;

    for child in &node.children {
        let to_x = child.x + NODE_W / 2.0;
        let to_y = child.y;
        let diff = diff_actions(&node.actions, &child.actions);

        edges.push(Edge {
            from_x,
            from_y,
            to_x,
            to_y,
            diff_champions: diff,
        });
        edges.extend(collect_edges(child));
    }
    edges
}

/// Collect all positioned nodes (flat list for rendering)
fn collect_nodes(node: &LayoutNode) -> Vec<LayoutNode> {
    let mut result = vec![node.clone()];
    for child in &node.children {
        result.extend(collect_nodes(child));
    }
    result
}

/// Compute total canvas dimensions from layout
fn canvas_size(roots: &[LayoutNode]) -> (f64, f64) {
    fn max_bounds(node: &LayoutNode) -> (f64, f64) {
        let mut max_x = node.x + NODE_W;
        let mut max_y = node.y + NODE_H;
        for child in &node.children {
            let (cx, cy) = max_bounds(child);
            max_x = max_x.max(cx);
            max_y = max_y.max(cy);
        }
        (max_x, max_y)
    }

    let mut w = 0.0_f64;
    let mut h = 0.0_f64;
    for root in roots {
        let (rx, ry) = max_bounds(root);
        w = w.max(rx);
        h = h.max(ry);
    }
    (w + 20.0, h + 20.0) // padding
}

#[component]
pub fn TreeGraph(
    roots: Vec<DraftTreeNode>,
    selected_node_id: ReadSignal<Option<String>>,
    on_select: Callback<DraftTreeNode>,
    on_add_branch: Callback<String>,
    champion_map: StoredValue<std::collections::HashMap<String, Champion>>,
    /// Flat list of original nodes for select callback
    #[prop(into)]
    all_nodes: StoredValue<Vec<DraftTreeNode>>,
) -> impl IntoView {
    // Build and layout tree
    let mut layout_roots = to_layout_nodes(&roots);
    for root in &mut layout_roots {
        compute_widths(root);
    }

    // Position roots side by side
    let mut offset_x = 10.0;
    let start_y = 10.0;
    for root in &mut layout_roots {
        assign_positions(root, offset_x, start_y);
        offset_x += root.width + H_GAP * 2.0;
    }

    let (canvas_w, canvas_h) = canvas_size(&layout_roots);

    // Collect edges and nodes
    let mut all_edges = Vec::new();
    let mut all_positioned = Vec::new();
    for root in &layout_roots {
        all_edges.extend(collect_edges(root));
        all_positioned.extend(collect_nodes(root));
    }

    view! {
        <div class="w-full overflow-auto bg-elevated/30 border border-divider/30 rounded-xl" style="max-height: 70vh;">
            <svg
                width=format!("{canvas_w}")
                height=format!("{canvas_h}")
                viewBox=format!("0 0 {canvas_w} {canvas_h}")
                xmlns="http://www.w3.org/2000/svg"
                class="block"
            >
                // Edges
                {all_edges.into_iter().map(|edge| {
                    let mid_y = (edge.from_y + edge.to_y) / 2.0;
                    let path = format!(
                        "M {},{} C {},{} {},{} {},{}",
                        edge.from_x, edge.from_y,
                        edge.from_x, mid_y,
                        edge.to_x, mid_y,
                        edge.to_x, edge.to_y
                    );
                    let all_icons = edge.diff_champions.clone();
                    let icon_mid_x = (edge.from_x + edge.to_x) / 2.0;
                    let icon_mid_y = mid_y;
                    // Cap at 3 icons; show overflow count as text
                    const MAX_ICONS: usize = 3;
                    let overflow = all_icons.len().saturating_sub(MAX_ICONS);
                    let icons: Vec<_> = all_icons.into_iter().take(MAX_ICONS).collect();
                    let n = icons.len();
                    // Center the row: start_x so the group is centered at icon_mid_x
                    let row_width = n as f64 * ICON_SIZE + (n.saturating_sub(1)) as f64 * 2.0;
                    let row_start_x = icon_mid_x - row_width / 2.0;

                    view! {
                        <path
                            d=path
                            fill="none"
                            stroke="var(--t-divider)"
                            stroke-width="2"
                        />
                        // Champion icons on edge (centered, capped at MAX_ICONS)
                        {icons.into_iter().enumerate().map(|(i, diff)| {
                            let ix = row_start_x + i as f64 * (ICON_SIZE + 2.0);
                            let iy = icon_mid_y - ICON_SIZE / 2.0;
                            let image_url = champion_map.with_value(|m| {
                                m.get(&diff.name).map(|c| c.image_full.clone()).unwrap_or_default()
                            });
                            let border_color = if diff.is_ban {
                                "var(--t-accent, #ef4444)"
                            } else {
                                "var(--t-accent, #22c55e)"
                            };
                            view! {
                                <g>
                                    // Background circle
                                    <circle
                                        cx=format!("{}", ix + ICON_SIZE / 2.0)
                                        cy=format!("{}", iy + ICON_SIZE / 2.0)
                                        r=format!("{}", ICON_SIZE / 2.0 + 2.0)
                                        fill="var(--t-elevated)"
                                        stroke=border_color
                                        stroke-width="1.5"
                                    />
                                    // Ban cross overlay
                                    {diff.is_ban.then(|| {
                                        let cx = ix + ICON_SIZE / 2.0;
                                        let cy = iy + ICON_SIZE / 2.0;
                                        let r = ICON_SIZE / 2.0;
                                        view! {
                                            <line
                                                x1=format!("{}", cx - r * 0.6)
                                                y1=format!("{}", cy - r * 0.6)
                                                x2=format!("{}", cx + r * 0.6)
                                                y2=format!("{}", cy + r * 0.6)
                                                stroke="#ef4444"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                            />
                                        }
                                    })}
                                    // Champion image (clipped to circle)
                                    <clipPath id=format!("clip-edge-{}-{}", ix as i32, iy as i32)>
                                        <circle
                                            cx=format!("{}", ix + ICON_SIZE / 2.0)
                                            cy=format!("{}", iy + ICON_SIZE / 2.0)
                                            r=format!("{}", ICON_SIZE / 2.0)
                                        />
                                    </clipPath>
                                    <image
                                        href=image_url
                                        x=format!("{ix}")
                                        y=format!("{iy}")
                                        width=format!("{ICON_SIZE}")
                                        height=format!("{ICON_SIZE}")
                                        clip-path=format!("url(#clip-edge-{}-{})", ix as i32, iy as i32)
                                        preserveAspectRatio="xMidYMid slice"
                                    />
                                </g>
                            }
                        }).collect_view()}
                        // Overflow badge: "+N more"
                        {(overflow > 0).then(|| {
                            let bx = row_start_x + n as f64 * (ICON_SIZE + 2.0);
                            let by = icon_mid_y;
                            view! {
                                <text
                                    x=format!("{}", bx + 2.0)
                                    y=format!("{}", by + ICON_SIZE * 0.7)
                                    font-size="9"
                                    fill="var(--t-muted)"
                                >{format!("+{overflow}")}</text>
                            }
                        })}
                    }
                }).collect_view()}

                // Nodes
                {all_positioned.into_iter().map(|n| {
                    let nid = n.id.clone();
                    let nid_for_click = nid.clone();
                    let nid_for_add = nid.clone();
                    let nid_for_selected = nid.clone();
                    let label = n.label.clone();
                    let is_improvised = n.is_improvised;
                    let is_root = n.is_root;
                    let x = n.x;
                    let y = n.y;
                    let action_count = n.actions.len();

                    view! {
                        <g
                            class="cursor-pointer"
                            on:click=move |_| {
                                let nid = nid_for_click.clone();
                                all_nodes.with_value(|nodes| {
                                    fn find_node(nodes: &[DraftTreeNode], id: &str) -> Option<DraftTreeNode> {
                                        for n in nodes {
                                            if n.id.as_deref() == Some(id) {
                                                return Some(n.clone());
                                            }
                                            if let Some(found) = find_node(&n.children, id) {
                                                return Some(found);
                                            }
                                        }
                                        None
                                    }
                                    if let Some(node) = find_node(nodes, &nid) {
                                        on_select.run(node);
                                    }
                                });
                            }
                        >
                            // Node rectangle
                            <rect
                                x=format!("{x}")
                                y=format!("{y}")
                                width=format!("{NODE_W}")
                                height=format!("{NODE_H}")
                                rx="8"
                                ry="8"
                                fill={
                                    let selected = selected_node_id.get_untracked().as_deref() == Some(&nid_for_selected);
                                    if selected { "var(--t-accent)" } else { "var(--t-elevated)" }
                                }
                                stroke={
                                    let selected = selected_node_id.get_untracked().as_deref() == Some(&nid_for_selected);
                                    if selected { "var(--t-accent-hover)" } else { "var(--t-divider)" }
                                }
                                stroke-width="1.5"
                            />

                            // Icon indicator
                            <text
                                x=format!("{}", x + 10.0)
                                y=format!("{}", y + NODE_H / 2.0 + 4.0)
                                font-size="10"
                                fill={
                                    if is_improvised { "#fbbf24" }
                                    else if is_root { "#34d399" }
                                    else { "var(--t-dimmed)" }
                                }
                            >
                                {if is_improvised { "\u{26A1}" }
                                 else if is_root { "\u{25C9}" }
                                 else { "\u{251C}" }}
                            </text>

                            // Label
                            <text
                                x=format!("{}", x + 22.0)
                                y=format!("{}", y + NODE_H / 2.0 + 4.0)
                                font-size="11"
                                fill={
                                    let selected = selected_node_id.get_untracked().as_deref() == Some(&nid);
                                    if selected { "var(--t-accent-contrast)" } else { "var(--t-primary)" }
                                }
                                class="select-none"
                            >
                                // Truncate label to fit
                                {if label.len() > 14 {
                                    format!("{}...", &label[..12])
                                } else {
                                    label.clone()
                                }}
                            </text>

                            // Action count badge
                            {(action_count > 0).then(|| {
                                let badge_x = x + NODE_W - 24.0;
                                let badge_y = y + 6.0;
                                view! {
                                    <rect
                                        x=format!("{badge_x}")
                                        y=format!("{badge_y}")
                                        width="18"
                                        height="14"
                                        rx="7"
                                        fill="var(--t-overlay-strong)"
                                    />
                                    <text
                                        x=format!("{}", badge_x + 9.0)
                                        y=format!("{}", badge_y + 10.5)
                                        font-size="9"
                                        text-anchor="middle"
                                        fill="var(--t-secondary)"
                                    >
                                        {action_count.to_string()}
                                    </text>
                                }
                            })}

                            // Add branch button (small + on right edge)
                            <g
                                class="cursor-pointer"
                                on:click=move |ev: web_sys::MouseEvent| {
                                    ev.stop_propagation();
                                    on_add_branch.run(nid_for_add.clone());
                                }
                            >
                                <circle
                                    cx=format!("{}", x + NODE_W + 2.0)
                                    cy=format!("{}", y + NODE_H / 2.0)
                                    r="8"
                                    fill="var(--t-overlay)"
                                    class="opacity-0 hover:opacity-100 transition-opacity"
                                />
                                <text
                                    x=format!("{}", x + NODE_W + 2.0)
                                    y=format!("{}", y + NODE_H / 2.0 + 4.0)
                                    font-size="12"
                                    text-anchor="middle"
                                    fill="var(--t-accent)"
                                    class="opacity-0 hover:opacity-100 transition-opacity"
                                >
                                    "+"
                                </text>
                            </g>
                        </g>
                    }
                }).collect_view()}
            </svg>
        </div>
    }
}
