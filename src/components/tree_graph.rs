use crate::models::champion::Champion;
use crate::models::draft::{DraftAction, DraftTreeNode};
use leptos::prelude::*;

// ----------------------------------------------------------------------------
// Phase 17 visual restyle.
//
// PRESERVED behaviour (per CLAUDE.md rule 41 + .planning/phases/17-ui-consolidation/
// 17-PATTERNS.md):
//   - LayoutNode + the recursive layout algorithm (`compute_widths`,
//     `assign_positions`, `to_layout_nodes`) are untouched.
//   - Tree assembly is performed upstream in `src/server/db.rs` via the
//     `children_of: HashMap<String, Vec<String>>` DFS — this component only
//     paints the nodes it receives.
//
// ADDED visuals (per 17-UI-SPEC §"Tree Graph Interactions"):
//   - 5 logical node states (locked / selected / alternate / ghost / leaf).
//     `selected` is reactive (driven by `selected_node_id`); the remaining
//     defaults are derived per node:
//       * `leaf`     — node has no children
//       * `ghost`    — improvised node (faint, dashed)
//       * `locked`   — has populated `actions`
//       * `alternate`— default fallback (regular node)
//   - Animated dash-flow on selected edges via an inline `<style>` block (the
//     `@keyframes dashFlow` rule is scoped to this component to avoid leaking
//     into global CSS).
//   - SVG strokes use `style="stroke: var(--color-…)"` because Tailwind
//     utilities don't reach SVG `stroke`. CSS custom properties are
//     re-evaluated by the browser on theme switch (see 17-RESEARCH.md
//     Pitfall 9).
//
// G-12 / raw-hex compliance: every interactive overlay uses `cursor-pointer`
// + reactive `var(--color-…)` tokens; no raw hex colours. The "+" overlay
// remains visually subtle (opacity-driven hover) — not a focusable element
// in the SVG context, so a focus ring is intentionally absent.
// ----------------------------------------------------------------------------

// Layout constants
const NODE_W: f64 = 180.0;
const NODE_H: f64 = 56.0;
const LEVEL_H: f64 = 120.0;
const H_GAP: f64 = 24.0;
const ICON_SIZE: f64 = 26.0;

/// Logical node state per 17-UI-SPEC §"Tree Graph Interactions" → 5-row table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeState {
    Locked,
    Alternate,
    Ghost,
    Leaf,
}

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

impl LayoutNode {
    /// Default state when this node is NOT the actively-selected one. The
    /// `selected` state is applied reactively at render time.
    fn default_state(&self) -> NodeState {
        if self.children.is_empty() {
            NodeState::Leaf
        } else if self.is_improvised {
            NodeState::Ghost
        } else if !self.actions.is_empty() {
            NodeState::Locked
        } else {
            NodeState::Alternate
        }
    }
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
    /// Logical state of the *destination* node — drives stroke style.
    to_state: NodeState,
    /// Owning child node id (for selected-edge highlight).
    to_id: String,
    /// Side of the new pick/ban (`us` = our team, `them` = opponent). `None`
    /// when the edge has no diff actions.
    side: Option<EdgeSide>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeSide {
    Us,
    Them,
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
    let children_width: f64 = node.children.iter().map(|c| c.width).sum::<f64>()
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

    let children_total: f64 = node.children.iter().map(|c| c.width).sum::<f64>()
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

/// Determine which side the diff actions belong to. Returns `Some(side)` when
/// every diff action shares the same `side` field; `None` otherwise.
fn diff_side(parent: &[DraftAction], child: &[DraftAction]) -> Option<EdgeSide> {
    let parent_set: std::collections::HashSet<(i32, &str)> = parent
        .iter()
        .map(|a| (a.order, a.champion.as_str()))
        .collect();

    let mut seen: Option<EdgeSide> = None;
    for action in child {
        if action.champion.is_empty() {
            continue;
        }
        if parent_set.contains(&(action.order, action.champion.as_str())) {
            continue;
        }
        let side = match action.side.as_str() {
            // "us" / "blue" / "ally" all map to Us; everything else is Them.
            // The Phase 12 schema settled on side = "blue" or "red" — treat
            // "blue" as our team to match the pre-existing draft board tint
            // convention (info = lapis blue).
            "blue" | "us" | "ally" => EdgeSide::Us,
            _ => EdgeSide::Them,
        };
        match seen {
            Some(prev) if prev != side => return None,
            _ => seen = Some(side),
        }
    }
    seen
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
        let side = diff_side(&node.actions, &child.actions);

        edges.push(Edge {
            from_x,
            from_y,
            to_x,
            to_y,
            diff_champions: diff,
            to_state: child.default_state(),
            to_id: child.id.clone(),
            side,
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

/// Build the `style` attribute for a non-selected edge based on its
/// destination state + side. Selected edges render a separate animated
/// stroke on top of this base stroke.
fn edge_base_style(state: NodeState, side: Option<EdgeSide>) -> &'static str {
    match (state, side) {
        // Ghost lineage: faint, dashed-feel, regardless of side.
        (NodeState::Ghost, _) => {
            "stroke: var(--color-muted); stroke-width: 1; opacity: 0.25"
        }
        // Side-tinted edges (when diff actions exist).
        (_, Some(EdgeSide::Us)) => {
            "stroke: var(--color-info); stroke-width: 1.5; opacity: 0.7"
        }
        (_, Some(EdgeSide::Them)) => {
            "stroke: var(--color-danger); stroke-width: 1.5; opacity: 0.7"
        }
        // Alternate / fallback.
        _ => "stroke: var(--color-secondary); stroke-width: 1.5; opacity: 0.6",
    }
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
        <div class="w-full overflow-auto bg-elevated/30 border border-divider/30 rounded-xl" style="max-height: 85vh;">
            <svg
                width=format!("{canvas_w}")
                height=format!("{canvas_h}")
                viewBox=format!("0 0 {canvas_w} {canvas_h}")
                xmlns="http://www.w3.org/2000/svg"
                class="block"
            >
                // Inline style: dash-flow keyframe + selected-state animation. Scoped
                // to this component to avoid leaking globally and keep CSS-variable
                // re-evaluation working across themes (Pitfall 9).
                <style>
                    {r#"
                    @keyframes dashFlow {
                        from { stroke-dashoffset: 0; }
                        to { stroke-dashoffset: -20; }
                    }
                    .tree-edge-selected {
                        animation: dashFlow 1.2s linear infinite;
                    }
                    "#}
                </style>
                // SVG filters for selected node halo + ban grayscale.
                <defs>
                    <filter id="selected-glow" x="-30%" y="-30%" width="160%" height="160%">
                        <feGaussianBlur in="SourceAlpha" stdDeviation="4" result="blur" />
                        <feFlood flood-color="var(--color-accent)" flood-opacity="0.5" result="color" />
                        <feComposite in="color" in2="blur" operator="in" result="glow" />
                        <feMerge>
                            <feMergeNode in="glow" />
                            <feMergeNode in="SourceGraphic" />
                        </feMerge>
                    </filter>
                    <filter id="grayscale-ban" color-interpolation-filters="sRGB">
                        <feColorMatrix type="saturate" values="0"/>
                    </filter>
                </defs>
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
                    let path_for_selected = path.clone();
                    let to_id_for_style = edge.to_id.clone();
                    let to_id_for_selected_class = edge.to_id.clone();
                    let to_id_for_selected_style = edge.to_id.clone();
                    let to_state = edge.to_state;
                    let side = edge.side;
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
                        // Base stroke (state-driven). Reactive opacity dims the base
                        // when this edge's destination is the selected node so the
                        // animated overlay reads cleanly.
                        <path
                            d=path
                            fill="none"
                            stroke-dasharray={
                                if matches!(to_state, NodeState::Ghost) { "4 4" } else { "" }
                            }
                            style=move || {
                                let base = edge_base_style(to_state, side);
                                if selected_node_id.get().as_deref() == Some(to_id_for_style.as_str()) {
                                    format!("{base}; opacity: 0.18")
                                } else {
                                    base.to_string()
                                }
                            }
                        />
                        // Selected overlay: animated dash, accent stroke, only visible
                        // when this edge points at the currently-selected node.
                        <path
                            d=path_for_selected
                            fill="none"
                            stroke-dasharray="6 4"
                            class=move || {
                                if selected_node_id.get().as_deref() == Some(to_id_for_selected_class.as_str()) {
                                    "tree-edge-selected"
                                } else {
                                    ""
                                }
                            }
                            style=move || {
                                if selected_node_id.get().as_deref() == Some(to_id_for_selected_style.as_str()) {
                                    "stroke: var(--color-accent); stroke-width: 2"
                                } else {
                                    "stroke: transparent; stroke-width: 0"
                                }
                            }
                        />
                        // Champion icons on edge (centered, capped at MAX_ICONS)
                        {icons.into_iter().enumerate().map(|(i, diff)| {
                            let ix = row_start_x + i as f64 * (ICON_SIZE + 2.0);
                            let iy = icon_mid_y - ICON_SIZE / 2.0;
                            let image_url = champion_map.with_value(|m| {
                                m.get(&diff.name).map(|c| c.image_full.clone()).unwrap_or_default()
                            });
                            let border_style = if diff.is_ban {
                                "stroke: var(--color-muted)"
                            } else {
                                "stroke: var(--color-accent)"
                            };
                            view! {
                                <g>
                                    // Background circle
                                    <circle
                                        cx=format!("{}", ix + ICON_SIZE / 2.0)
                                        cy=format!("{}", iy + ICON_SIZE / 2.0)
                                        r=format!("{}", ICON_SIZE / 2.0 + 2.0)
                                        style=format!("fill: var(--color-elevated); {}; stroke-width: 1.5", border_style)
                                    />
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
                                        filter={if diff.is_ban { "url(#grayscale-ban)" } else { "" }}
                                        opacity={if diff.is_ban { "0.5" } else { "1" }}
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
                                    style="fill: var(--color-muted)"
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
                    let nid_for_rect_style = nid.clone();
                    let nid_for_rect_filter = nid.clone();
                    let nid_for_rect_dasharray = nid.clone();
                    let nid_for_label_style = nid.clone();
                    let label = n.label.clone();
                    let is_improvised = n.is_improvised;
                    let is_root = n.is_root;
                    let x = n.x;
                    let y = n.y;
                    let action_count = n.actions.len();
                    let default_state = n.default_state();
                    // Leaf rendering uses a circular tile centred on the rect's
                    // mid-point. We still draw the rect underneath so layout/edge
                    // anchors stay aligned with the existing algorithm.
                    let is_leaf = matches!(default_state, NodeState::Leaf);
                    let leaf_cx = x + NODE_W / 2.0;
                    let leaf_cy = y + NODE_H / 2.0;

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
                            // Node rectangle — reactive style based on selected_node_id +
                            // default state. State styles per UI-SPEC §"Tree Graph".
                            <rect
                                x=format!("{x}")
                                y=format!("{y}")
                                width=format!("{NODE_W}")
                                height=format!("{NODE_H}")
                                rx={if is_leaf { format!("{}", NODE_H / 2.0) } else { "10".to_string() }}
                                ry={if is_leaf { format!("{}", NODE_H / 2.0) } else { "10".to_string() }}
                                stroke-dasharray=move || {
                                    let selected = selected_node_id.get().as_deref()
                                        == Some(nid_for_rect_dasharray.as_str());
                                    if selected {
                                        ""
                                    } else {
                                        match default_state {
                                            NodeState::Ghost => "4 4",
                                            _ => "",
                                        }
                                    }
                                }
                                style=move || {
                                    let selected = selected_node_id.get().as_deref()
                                        == Some(nid_for_rect_style.as_str());
                                    if selected {
                                        // Selected state: gold halo via filter +
                                        // accent ring (stroke-width 2.5).
                                        "fill: var(--color-accent); stroke: var(--color-accent-hover); stroke-width: 2.5; opacity: 1".to_string()
                                    } else {
                                        match default_state {
                                            NodeState::Locked => {
                                                "fill: var(--color-elevated); stroke: var(--color-outline); stroke-width: 1.5; opacity: 1".to_string()
                                            }
                                            NodeState::Alternate => {
                                                "fill: var(--color-elevated); stroke: var(--color-divider); stroke-width: 1.5; opacity: 0.78".to_string()
                                            }
                                            NodeState::Ghost => {
                                                "fill: var(--color-elevated); stroke: var(--color-divider); stroke-width: 1; opacity: 0.3".to_string()
                                            }
                                            NodeState::Leaf => {
                                                // Leaf: subtle ring; outer fill set on the
                                                // overlay circle below.
                                                "fill: var(--color-elevated); stroke: var(--color-accent); stroke-width: 1.5; opacity: 0.9".to_string()
                                            }
                                        }
                                    }
                                }
                                filter=move || {
                                    if selected_node_id.get().as_deref() == Some(nid_for_rect_filter.as_str()) {
                                        "url(#selected-glow)"
                                    } else {
                                        ""
                                    }
                                }
                            />

                            // Leaf overlay: a glowing aureole. Only visible when this
                            // node is a leaf (no children).
                            {is_leaf.then(|| view! {
                                <circle
                                    cx=format!("{leaf_cx}")
                                    cy=format!("{leaf_cy}")
                                    r="22"
                                    style="fill: var(--color-accent-soft); stroke: var(--color-accent); stroke-width: 1.5; opacity: 0.6"
                                />
                            })}

                            // Phase / role indicator (small glyph on the left).
                            <text
                                x=format!("{}", x + 12.0)
                                y=format!("{}", y + NODE_H / 2.0 + 4.0)
                                font-size="11"
                                style={
                                    if is_improvised {
                                        "fill: var(--color-warning)"
                                    } else if is_root {
                                        "fill: var(--color-success)"
                                    } else {
                                        "fill: var(--color-dimmed)"
                                    }
                                }
                            >
                                {if is_improvised { "\u{26A1}" }
                                 else if is_root { "\u{25C9}" }
                                 else { "\u{251C}" }}
                            </text>

                            // Label — reactive fill based on selection.
                            <text
                                x=format!("{}", x + 28.0)
                                y=format!("{}", y + NODE_H / 2.0 + 4.0)
                                font-size="13"
                                font-weight="600"
                                style=move || {
                                    if selected_node_id.get().as_deref() == Some(nid_for_label_style.as_str()) {
                                        "fill: var(--color-accent-contrast)"
                                    } else {
                                        match default_state {
                                            NodeState::Alternate => "fill: var(--color-secondary)",
                                            NodeState::Ghost => "fill: var(--color-muted)",
                                            _ => "fill: var(--color-primary)",
                                        }
                                    }
                                }
                                class="select-none"
                            >
                                // Truncate label to fit
                                {if label.len() > 18 {
                                    format!("{}...", &label[..16])
                                } else {
                                    label.clone()
                                }}
                            </text>

                            // Action count badge (picks/bans on this node).
                            {(action_count > 0).then(|| {
                                let badge_x = x + NODE_W - 32.0;
                                let badge_y = y + 8.0;
                                view! {
                                    <rect
                                        x=format!("{badge_x}")
                                        y=format!("{badge_y}")
                                        width="22"
                                        height="16"
                                        rx="7"
                                        style="fill: var(--color-overlay-strong)"
                                    />
                                    <text
                                        x=format!("{}", badge_x + 11.0)
                                        y=format!("{}", badge_y + 12.0)
                                        font-size="9"
                                        text-anchor="middle"
                                        style="fill: var(--color-secondary)"
                                    >
                                        {action_count.to_string()}
                                    </text>
                                }
                            })}

                            // Add branch button (small + on right edge), suppressed
                            // for ghost nodes per UI-SPEC.
                            {(!matches!(default_state, NodeState::Ghost)).then(|| view! {
                                <g
                                    class="cursor-pointer"
                                    on:click=move |ev: web_sys::MouseEvent| {
                                        ev.stop_propagation();
                                        on_add_branch.run(nid_for_add.clone());
                                    }
                                >
                                    <circle
                                        cx=format!("{}", x + NODE_W + 4.0)
                                        cy=format!("{}", y + NODE_H / 2.0)
                                        r="9"
                                        style="fill: var(--color-overlay)"
                                        class="opacity-0 hover:opacity-100 transition-opacity"
                                    />
                                    <text
                                        x=format!("{}", x + NODE_W + 4.0)
                                        y=format!("{}", y + NODE_H / 2.0 + 5.0)
                                        font-size="13"
                                        text-anchor="middle"
                                        style="fill: var(--color-accent)"
                                        class="opacity-0 hover:opacity-100 transition-opacity"
                                    >
                                        "+"
                                    </text>
                                </g>
                            })}
                        </g>
                    }
                }).collect_view()}
            </svg>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::draft::DraftAction;

    fn action(side: &str, kind: &str, order: i32, champ: &str) -> DraftAction {
        DraftAction {
            id: None,
            draft_id: String::new(),
            phase: format!("{}_{}", kind, order),
            side: side.to_string(),
            champion: champ.to_string(),
            order,
            comment: None,
            role: None,
        }
    }

    fn leaf(id: &str) -> LayoutNode {
        LayoutNode {
            id: id.to_string(),
            label: id.to_string(),
            is_improvised: false,
            is_root: false,
            actions: Vec::new(),
            children: Vec::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
        }
    }

    #[test]
    fn default_state_leaf_when_no_children() {
        let n = leaf("a");
        assert_eq!(n.default_state(), NodeState::Leaf);
    }

    #[test]
    fn default_state_ghost_when_improvised() {
        let mut n = leaf("a");
        n.is_improvised = true;
        n.children = vec![leaf("b")];
        assert_eq!(n.default_state(), NodeState::Ghost);
    }

    #[test]
    fn default_state_locked_when_actions() {
        let mut n = leaf("a");
        n.children = vec![leaf("b")];
        n.actions = vec![action("blue", "pick", 0, "Aatrox")];
        assert_eq!(n.default_state(), NodeState::Locked);
    }

    #[test]
    fn default_state_alternate_otherwise() {
        let mut n = leaf("a");
        n.children = vec![leaf("b")];
        assert_eq!(n.default_state(), NodeState::Alternate);
    }

    #[test]
    fn diff_side_us_for_blue_picks() {
        let parent: Vec<DraftAction> = Vec::new();
        let child = vec![action("blue", "pick", 0, "Aatrox")];
        assert_eq!(diff_side(&parent, &child), Some(EdgeSide::Us));
    }

    #[test]
    fn diff_side_them_for_red_picks() {
        let parent: Vec<DraftAction> = Vec::new();
        let child = vec![action("red", "pick", 0, "Aatrox")];
        assert_eq!(diff_side(&parent, &child), Some(EdgeSide::Them));
    }

    #[test]
    fn diff_side_none_when_mixed() {
        let parent: Vec<DraftAction> = Vec::new();
        let child = vec![
            action("blue", "pick", 0, "Aatrox"),
            action("red", "pick", 1, "Brand"),
        ];
        assert_eq!(diff_side(&parent, &child), None);
    }

    #[test]
    fn diff_side_none_when_no_diff() {
        let parent = vec![action("blue", "pick", 0, "Aatrox")];
        let child = vec![action("blue", "pick", 0, "Aatrox")];
        assert_eq!(diff_side(&parent, &child), None);
    }

    #[test]
    fn edge_base_style_uses_only_var_tokens() {
        // Sanity: no raw hex in any base-edge style branch (Pitfall 9).
        let cases = [
            (NodeState::Locked, None),
            (NodeState::Alternate, None),
            (NodeState::Ghost, None),
            (NodeState::Leaf, None),
            (NodeState::Locked, Some(EdgeSide::Us)),
            (NodeState::Locked, Some(EdgeSide::Them)),
            (NodeState::Ghost, Some(EdgeSide::Us)),
        ];
        for (state, side) in cases {
            let s = edge_base_style(state, side);
            assert!(
                s.contains("var(--color-"),
                "edge style for ({state:?}, {side:?}) lacks CSS var: {s}"
            );
            assert!(
                !s.contains('#'),
                "edge style for ({state:?}, {side:?}) contains raw hex: {s}"
            );
        }
    }
}
