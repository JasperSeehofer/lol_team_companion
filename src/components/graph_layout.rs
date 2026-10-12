//! Force-directed layout of the Draft Graph canvas (M4 stage B).
//!
//! Pure Rust, no browser types, shared by both targets and unit-tested:
//! the canvas (`components::graph_canvas`) only drives it frame by frame.
//!
//! The layout space is 800 x 500 units, a grid of eight columns (A to H)
//! by five rows (1 to 5) of 100 units each, the same grid the
//! `CanvasFrame` draws, so a position reads as a grid reference ("D4").
//!
//! - **Pinned nodes**: the locked picks. The ranked side sits in column A,
//!   the other side in column H, one row per role (Top in row 1 down to
//!   Support in row 5), so the pins never move and always read the same.
//! - **Free nodes**: the top candidates (at most `CANVAS_CANDIDATES`). Each
//!   is pulled toward the picks it has synergy with (and, more weakly,
//!   toward the enemy picks it beats), pushed away from the picks that beat
//!   it or work against it, kept apart from the other nodes (a short-range
//!   charge plus a collision radius that grows with its contour rings) and
//!   held by a gentle pull to the middle that is stronger for a higher fit,
//!   so the summits sit near the centre and weak candidates drift outward.
//! - **Integration**: velocity Verlet with damping and a cooling `alpha`
//!   (the d3-force scheme): every force is scaled by `alpha`, which decays
//!   from 1 to [`ALPHA_MIN`] in about 150 ticks; the simulation also stops
//!   when nothing moves any more. After that the layout is frozen: the
//!   canvas stops ticking (no perpetual motion). A drag pins one node and
//!   reheats the rest; on release everything settles again.
//! - **Deterministic**: the start positions come from the rank (a golden
//!   angle spiral around the centre) or from the previous layout of the
//!   same champion, so the same draft always draws the same picture.

use std::collections::HashMap;

use crate::models::graph::{GraphEdge, GraphNode, GraphResult, RelationKind, Role};

/// Width of the layout space in units.
pub const WIDTH: f32 = 800.0;
/// Height of the layout space in units.
pub const HEIGHT: f32 = 500.0;
/// Grid columns (A to H) and rows (1 to 5).
pub const COLS: u8 = 8;
pub const ROWS: u8 = 5;
/// Units per grid cell.
pub const CELL: f32 = 100.0;
/// Column centres of the pinned sides (A and H).
pub const PIN_LEFT_X: f32 = 50.0;
pub const PIN_RIGHT_X: f32 = 750.0;
/// Where free nodes may go (columns B to G, a margin from the edges).
pub const FREE_MIN_X: f32 = 140.0;
pub const FREE_MAX_X: f32 = 660.0;
pub const FREE_MIN_Y: f32 = 30.0;
pub const FREE_MAX_Y: f32 = 470.0;
/// The simulation stops below this alpha.
pub const ALPHA_MIN: f32 = 0.005;
/// Ticks for alpha to fall from 1 to `ALPHA_MIN`.
pub const COOLING_TICKS: f32 = 150.0;
/// Hard cap on ticks per run (a reheat starts a new run).
pub const MAX_TICKS: u32 = 400;
/// Velocity kept per tick is `1 - VELOCITY_DECAY`.
pub const VELOCITY_DECAY: f32 = 0.4;
/// Short-range repulsion between nodes.
pub const CHARGE: f32 = 320.0;
/// The charge is ignored beyond this distance.
pub const CHARGE_RANGE: f32 = 220.0;
/// Below this speed (units per tick) for every free node, the layout has
/// settled even if alpha has not run out.
pub const REST_SPEED: f32 = 0.02;
/// Contour rings start at this fit and come every `RING_STEP` points.
pub const RING_FROM: u8 = 50;
pub const RING_STEP: u8 = 10;

/// The grid reference of a point: "A1" (top left) to "H5" (bottom right).
pub fn grid_ref(x: f32, y: f32) -> String {
    let col = ((x / CELL).floor().max(0.0) as u8).min(COLS - 1);
    let row = ((y / CELL).floor().max(0.0) as u8).min(ROWS - 1);
    format!("{}{}", char::from(b'A' + col), row + 1)
}

/// Contour rings for a fit: one at 50, 60, 70 ... up to the fit (none
/// below 50). Fit 84 has four rings, fit 100 six.
pub fn ring_count(fit: u8) -> u8 {
    if fit < RING_FROM {
        0
    } else {
        (fit - RING_FROM) / RING_STEP + 1
    }
}

/// The row a role's pin sits in (Top row 1 ... Support row 5), 0-based.
pub fn role_row(r: Role) -> usize {
    match r {
        Role::Top => 0,
        Role::Jungle => 1,
        Role::Mid => 2,
        Role::Bot => 3,
        Role::Support => 4,
    }
}

/// Rows for one side's pins: by role where the role is known and its row
/// is free, the rest in slot order into the remaining rows.
pub fn pin_rows(nodes: &[&GraphNode]) -> Vec<usize> {
    let mut taken = [false; ROWS as usize];
    let mut rows = vec![usize::MAX; nodes.len()];
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by_key(|&i| nodes[i].slot);
    for &i in &order {
        if let Some(r) = nodes[i].role.map(role_row) {
            if !taken[r] {
                taken[r] = true;
                rows[i] = r;
            }
        }
    }
    for &i in &order {
        if rows[i] == usize::MAX {
            let free = (0..ROWS as usize).find(|r| !taken[*r]);
            let r = free.unwrap_or(i % ROWS as usize);
            if r < taken.len() {
                taken[r] = true;
            }
            rows[i] = r;
        }
    }
    rows
}

/// Centre of a row in units.
pub fn row_y(row: usize) -> f32 {
    row as f32 * CELL + CELL / 2.0
}

/// How an edge moves its candidate: attract with a spring of stiffness
/// `k` toward `rest` units, or repel (a spring that only pushes, inside
/// `rest`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkForce {
    pub attract: bool,
    pub k: f32,
    pub rest: f32,
}

/// The force an edge puts on its candidate. Synergy pulls (stiffer the
/// stronger it is); a counter the candidate wins pulls more weakly and
/// from further away (the answer drifts toward the threat); a counter the
/// candidate loses, or a synergy that works against it, pushes away.
pub fn link_force(e: &GraphEdge) -> Option<LinkForce> {
    let s = e.strength.clamp(-4.0, 4.0);
    if s.abs() < 0.01 {
        return None;
    }
    Some(match (e.kind, s > 0.0) {
        (RelationKind::Synergy, true) => LinkForce {
            attract: true,
            k: 0.035 * s,
            rest: 190.0,
        },
        (RelationKind::Counter, true) => LinkForce {
            attract: true,
            k: 0.018 * s,
            rest: 230.0,
        },
        (_, false) => LinkForce {
            attract: false,
            k: 0.02 * s.abs(),
            rest: 520.0,
        },
    })
}

/// What a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// A locked pick; `left` for the ranked side.
    Pinned { left: bool },
    /// A candidate.
    Free,
}

/// One node of the simulation, in layout units.
#[derive(Clone, Debug, PartialEq)]
pub struct SimNode {
    pub id: String,
    pub kind: NodeKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// Held in place (pins always; a candidate while it is dragged).
    pub fixed: bool,
    /// Collision radius in units.
    pub radius: f32,
    /// Strength of the pull to the middle.
    pub pull: f32,
}

/// A spring between a free node and a pinned one (indices into `nodes`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimLink {
    pub free: usize,
    pub pinned: usize,
    pub force: LinkForce,
}

/// The simulation state.
#[derive(Clone, Debug, PartialEq)]
pub struct Simulation {
    pub nodes: Vec<SimNode>,
    pub links: Vec<SimLink>,
    pub alpha: f32,
    /// Ticks in the current run.
    pub ticks: u32,
    resting: bool,
}

const CENTRE: (f32, f32) = (WIDTH / 2.0, HEIGHT / 2.0);

impl Simulation {
    pub fn new(nodes: Vec<SimNode>, links: Vec<SimLink>, alpha: f32) -> Self {
        Self {
            nodes,
            links,
            alpha: alpha.clamp(0.0, 1.0),
            ticks: 0,
            resting: false,
        }
    }

    /// Alpha is multiplied by this every tick.
    fn cooling() -> f32 {
        ALPHA_MIN.powf(1.0 / COOLING_TICKS)
    }

    /// The layout has settled: alpha ran out, nothing moves, or the run hit
    /// the tick cap. A settled layout is frozen until [`reheat`](Self::reheat).
    pub fn is_settled(&self) -> bool {
        self.alpha < ALPHA_MIN || self.resting || self.ticks >= MAX_TICKS
    }

    /// Start moving again (after a drag or new data); alpha never drops.
    pub fn reheat(&mut self, alpha: f32) {
        self.alpha = self.alpha.max(alpha.min(1.0));
        self.ticks = 0;
        self.resting = false;
    }

    /// Hold node `i` at a point (dragging). Pins cannot be moved.
    pub fn fix(&mut self, i: usize, x: f32, y: f32) {
        if let Some(n) = self.nodes.get_mut(i) {
            if n.kind == NodeKind::Free {
                n.fixed = true;
                n.x = x.clamp(FREE_MIN_X, FREE_MAX_X);
                n.y = y.clamp(FREE_MIN_Y, FREE_MAX_Y);
                n.vx = 0.0;
                n.vy = 0.0;
            }
        }
    }

    /// Let a dragged node go.
    pub fn release(&mut self, i: usize) {
        if let Some(n) = self.nodes.get_mut(i) {
            if n.kind == NodeKind::Free {
                n.fixed = false;
            }
        }
    }

    /// One step. Does nothing once settled (the layout is frozen).
    pub fn tick(&mut self) {
        if self.is_settled() {
            return;
        }
        let alpha = self.alpha;
        let n = self.nodes.len();
        let mut fx = vec![0.0f32; n];
        let mut fy = vec![0.0f32; n];

        // Springs to the locked picks.
        for l in &self.links {
            let (a, b) = (&self.nodes[l.free], &self.nodes[l.pinned]);
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            let stretch = d - l.force.rest;
            if !l.force.attract && stretch >= 0.0 {
                continue;
            }
            let f = l.force.k * stretch * alpha / d;
            fx[l.free] += dx * f;
            fy[l.free] += dy * f;
        }

        // Short-range charge between every pair with a free node.
        for i in 0..n {
            for j in (i + 1)..n {
                let (a, b) = (&self.nodes[i], &self.nodes[j]);
                if a.kind != NodeKind::Free && b.kind != NodeKind::Free {
                    continue;
                }
                let mut dx = a.x - b.x;
                let mut dy = a.y - b.y;
                if dx.abs() < 1e-3 && dy.abs() < 1e-3 {
                    // Coincident: separate along a fixed, index-based direction.
                    let ang = (i * 7 + j * 13) as f32;
                    dx = ang.cos() * 0.5;
                    dy = ang.sin() * 0.5;
                }
                let d2 = (dx * dx + dy * dy).max(1.0);
                if d2 > CHARGE_RANGE * CHARGE_RANGE {
                    continue;
                }
                let w = CHARGE * alpha / d2;
                fx[i] += dx * w;
                fy[i] += dy * w;
                fx[j] -= dx * w;
                fy[j] -= dy * w;
            }
        }

        // Pull to the middle.
        for (i, node) in self.nodes.iter().enumerate() {
            if node.kind == NodeKind::Free {
                fx[i] += (CENTRE.0 - node.x) * node.pull * alpha;
                fy[i] += (CENTRE.1 - node.y) * node.pull * alpha;
            }
        }

        // Integrate.
        let keep = 1.0 - VELOCITY_DECAY;
        let mut fastest = 0.0f32;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.kind != NodeKind::Free || node.fixed {
                node.vx = 0.0;
                node.vy = 0.0;
                continue;
            }
            node.vx = (node.vx + fx[i]) * keep;
            node.vy = (node.vy + fy[i]) * keep;
            node.x += node.vx;
            node.y += node.vy;
            fastest = fastest.max(node.vx.abs()).max(node.vy.abs());
        }

        // Collisions: overlapping nodes are pushed apart (pins do not move).
        for i in 0..n {
            for j in (i + 1)..n {
                let (ki, kj) = (self.nodes[i].kind, self.nodes[j].kind);
                let mi = ki == NodeKind::Free && !self.nodes[i].fixed;
                let mj = kj == NodeKind::Free && !self.nodes[j].fixed;
                if !mi && !mj {
                    continue;
                }
                let dx = self.nodes[j].x - self.nodes[i].x;
                let dy = self.nodes[j].y - self.nodes[i].y;
                let min = self.nodes[i].radius + self.nodes[j].radius;
                let d2 = dx * dx + dy * dy;
                if d2 >= min * min {
                    continue;
                }
                let d = d2.sqrt().max(0.01);
                let push = (min - d) / d * 0.6;
                let (si, sj) = match (mi, mj) {
                    (true, true) => (0.5, 0.5),
                    (true, false) => (1.0, 0.0),
                    _ => (0.0, 1.0),
                };
                self.nodes[i].x -= dx * push * si;
                self.nodes[i].y -= dy * push * si;
                self.nodes[j].x += dx * push * sj;
                self.nodes[j].y += dy * push * sj;
            }
        }

        // Bounds.
        for node in self.nodes.iter_mut() {
            if node.kind != NodeKind::Free {
                continue;
            }
            let (x, y) = (
                node.x.clamp(FREE_MIN_X, FREE_MAX_X),
                node.y.clamp(FREE_MIN_Y, FREE_MAX_Y),
            );
            if x != node.x {
                node.vx = 0.0;
            }
            if y != node.y {
                node.vy = 0.0;
            }
            node.x = x;
            node.y = y;
        }

        self.alpha *= Self::cooling();
        self.ticks += 1;
        if self.ticks > 40 && fastest < REST_SPEED {
            self.resting = true;
        }
    }

    /// Tick until settled; returns the ticks taken.
    pub fn run(&mut self) -> u32 {
        let start = self.ticks;
        while !self.is_settled() {
            self.tick();
        }
        self.ticks - start
    }

    /// Node positions in units, in node order.
    pub fn positions(&self) -> Vec<(f32, f32)> {
        self.nodes.iter().map(|n| (n.x, n.y)).collect()
    }
}

/// A ghost marker for an open role of the ranked side (not a sim node).
#[derive(Clone, Debug, PartialEq)]
pub struct OpenRole {
    pub role: Role,
    pub x: f32,
    pub y: f32,
}

/// An edge as the canvas draws it: node indices plus the model edge.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedEdge {
    /// Index of the candidate's node.
    pub free: usize,
    /// Index of the locked pick's node.
    pub pinned: usize,
    pub edge: GraphEdge,
}

/// Everything the canvas needs for one `GraphResult`: the simulation (pins
/// first, then the candidates in rank order), the drawable edges and the
/// open roles of the ranked side.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutPlan {
    pub sim: Simulation,
    /// Number of pinned nodes (they come first in `sim.nodes`).
    pub pins: usize,
    pub edges: Vec<PlannedEdge>,
    pub open_roles: Vec<OpenRole>,
}

impl LayoutPlan {
    /// Index of a candidate's node by rank (0-based).
    pub fn free_index(&self, rank: usize) -> usize {
        self.pins + rank
    }
}

/// Build the layout for a result: at most `max_free` candidates; `radius`
/// gives a candidate's collision radius in units from its fit; `previous`
/// holds last positions by champion id (warm start).
pub fn plan(
    g: &GraphResult,
    max_free: usize,
    radius: impl Fn(u8) -> f32,
    pin_radius: f32,
    previous: &HashMap<String, (f32, f32)>,
) -> LayoutPlan {
    let mut nodes = Vec::new();
    for left in [true, false] {
        let side: Vec<&GraphNode> = g.locked.iter().filter(|n| n.own == left).collect();
        let rows = pin_rows(&side);
        for (n, row) in side.iter().zip(rows) {
            nodes.push(SimNode {
                id: n.champion.clone(),
                kind: NodeKind::Pinned { left },
                x: if left { PIN_LEFT_X } else { PIN_RIGHT_X },
                y: row_y(row),
                vx: 0.0,
                vy: 0.0,
                fixed: true,
                radius: pin_radius,
                pull: 0.0,
            });
        }
    }
    let pins = nodes.len();
    let mut carried = 0usize;
    let shown: Vec<_> = g.candidates.iter().take(max_free).collect();
    for (i, c) in shown.iter().enumerate() {
        let (x, y) = match previous.get(&c.champion) {
            Some(p) => {
                carried += 1;
                *p
            }
            None => spiral(i),
        };
        let lift = ((c.fit as f32 - 40.0) / 60.0).clamp(0.0, 1.0);
        nodes.push(SimNode {
            id: c.champion.clone(),
            kind: NodeKind::Free,
            x: x.clamp(FREE_MIN_X, FREE_MAX_X),
            y: y.clamp(FREE_MIN_Y, FREE_MAX_Y),
            vx: 0.0,
            vy: 0.0,
            fixed: false,
            radius: radius(c.fit),
            pull: 0.012 + 0.03 * lift,
        });
    }
    let index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut edges = Vec::new();
    let mut links = Vec::new();
    for e in &g.edges {
        let (Some(&free), Some(&pinned)) = (index.get(e.from.as_str()), index.get(e.to.as_str()))
        else {
            continue;
        };
        if free < pins || pinned >= pins {
            continue;
        }
        if let Some(force) = link_force(e) {
            links.push(SimLink {
                free,
                pinned,
                force,
            });
        }
        edges.push(PlannedEdge {
            free,
            pinned,
            edge: e.clone(),
        });
    }
    // Open roles of the ranked side, where no pin sits.
    let mut open_roles = Vec::new();
    if let Some(t) = &g.target {
        for r in &t.roles {
            let y = row_y(role_row(*r));
            let taken = nodes[..pins]
                .iter()
                .any(|n| n.kind == NodeKind::Pinned { left: true } && (n.y - y).abs() < 1.0);
            if !taken {
                open_roles.push(OpenRole {
                    role: *r,
                    x: PIN_LEFT_X,
                    y,
                });
            }
        }
    }
    // Mostly known positions: a gentle re-settle instead of a full one.
    let alpha = if !shown.is_empty() && carried * 2 >= shown.len() {
        0.5
    } else {
        1.0
    };
    LayoutPlan {
        sim: Simulation::new(nodes, links, alpha),
        pins,
        edges,
        open_roles,
    }
}

/// Start position by rank: a golden-angle spiral around the centre,
/// stretched to the canvas' proportions.
pub fn spiral(rank: usize) -> (f32, f32) {
    let a = rank as f32 * 2.399_963;
    let r = 30.0 + 34.0 * (rank as f32).sqrt();
    (CENTRE.0 + r * a.cos() * 1.3, CENTRE.1 + r * a.sin() * 0.85)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{Candidate, CompProfile, GraphTarget, ReasonSource};

    fn cand(id: &str, fit: u8) -> Candidate {
        Candidate {
            champion: id.into(),
            name: id.into(),
            fit,
            role: Some(Role::Support),
            role_guessed: true,
            parts: Vec::new(),
            reasons: Vec::new(),
            reasons_complete: true,
            pool: None,
            adds: CompProfile::default(),
            trait_default: true,
            trait_low_confidence: false,
        }
    }

    fn pin(id: &str, own: bool, slot: usize, role: Option<Role>) -> GraphNode {
        GraphNode {
            champion: id.into(),
            name: id.into(),
            side: if own { "blue" } else { "red" }.into(),
            slot,
            role,
            own,
        }
    }

    fn edge(from: &str, to: &str, kind: RelationKind, strength: f32) -> GraphEdge {
        GraphEdge {
            from: from.into(),
            to: to.into(),
            kind,
            strength,
            sources: vec![ReasonSource::Rule],
        }
    }

    /// A full mid-draft: four picks a side, 24 candidates, edges from most
    /// candidates to several picks.
    fn fixture() -> GraphResult {
        let roles = [Role::Top, Role::Jungle, Role::Mid, Role::Bot];
        let mut locked = Vec::new();
        for (i, r) in roles.iter().enumerate() {
            locked.push(pin(&format!("Own{i}"), true, 6 + i, Some(*r)));
            locked.push(pin(&format!("Foe{i}"), false, 7 + i, Some(*r)));
        }
        let candidates: Vec<Candidate> = (0..24)
            .map(|i| cand(&format!("C{i:02}"), 90 - i as u8 * 2))
            .collect();
        let mut edges = Vec::new();
        for (i, c) in candidates.iter().enumerate() {
            edges.push(edge(
                &c.champion,
                &format!("Own{}", i % 4),
                RelationKind::Synergy,
                1.0 + (i % 3) as f32,
            ));
            if i % 2 == 0 {
                edges.push(edge(
                    &c.champion,
                    &format!("Foe{}", (i + 1) % 4),
                    RelationKind::Counter,
                    if i % 4 == 0 { 2.0 } else { -2.0 },
                ));
            }
            if i % 5 == 0 {
                edges.push(edge(
                    &c.champion,
                    &format!("Own{}", (i + 2) % 4),
                    RelationKind::Synergy,
                    -1.0,
                ));
            }
        }
        GraphResult {
            target: Some(GraphTarget {
                slot: 17,
                side: "blue".into(),
                ours: true,
                roles: vec![Role::Support],
                role_chosen: false,
            }),
            candidates,
            locked,
            edges,
            ..GraphResult::default()
        }
    }

    fn radius(fit: u8) -> f32 {
        20.0 + 6.0 * ring_count(fit) as f32
    }

    fn settled(g: &GraphResult) -> LayoutPlan {
        let mut p = plan(g, 24, radius, 22.0, &HashMap::new());
        p.sim.run();
        p
    }

    fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
    }

    #[test]
    fn grid_references_read_like_the_frame() {
        assert_eq!(grid_ref(50.0, 50.0), "A1");
        assert_eq!(grid_ref(750.0, 450.0), "H5");
        assert_eq!(grid_ref(399.0, 260.0), "D3");
        assert_eq!(grid_ref(-5.0, 900.0), "A5");
        assert_eq!(grid_ref(800.0, 0.0), "H1");
    }

    #[test]
    fn rings_every_ten_fit_from_fifty() {
        assert_eq!(ring_count(49), 0);
        assert_eq!(ring_count(50), 1);
        assert_eq!(ring_count(71), 3);
        assert_eq!(ring_count(84), 4);
        assert_eq!(ring_count(100), 6);
    }

    #[test]
    fn pins_sit_in_their_columns_one_row_per_role() {
        let p = plan(&fixture(), 24, radius, 22.0, &HashMap::new());
        assert_eq!(p.pins, 8);
        for n in &p.sim.nodes[..p.pins] {
            match n.kind {
                NodeKind::Pinned { left: true } => assert_eq!(n.x, PIN_LEFT_X),
                NodeKind::Pinned { left: false } => assert_eq!(n.x, PIN_RIGHT_X),
                NodeKind::Free => panic!("pins come first"),
            }
        }
        // Own2 plays Mid: row 3, grid A3.
        let mid = p.sim.nodes.iter().find(|n| n.id == "Own2").unwrap();
        assert_eq!(grid_ref(mid.x, mid.y), "A3");
        // The open support role is a ghost in A5.
        assert_eq!(p.open_roles.len(), 1);
        assert_eq!(grid_ref(p.open_roles[0].x, p.open_roles[0].y), "A5");
    }

    #[test]
    fn pins_without_roles_fill_the_free_rows_in_slot_order() {
        let a = pin("A", true, 9, None);
        let b = pin("B", true, 6, Some(Role::Top));
        let c = pin("C", true, 7, Some(Role::Top));
        let rows = pin_rows(&[&a, &b, &c]);
        assert_eq!(
            rows,
            vec![2, 0, 1],
            "B keeps Top; C and A fill rows 2 and 3"
        );
    }

    #[test]
    fn the_layout_is_deterministic() {
        let a = settled(&fixture());
        let b = settled(&fixture());
        assert_eq!(a.sim.positions(), b.sim.positions());
    }

    #[test]
    fn it_settles_and_then_stays_frozen() {
        let mut p = plan(&fixture(), 24, radius, 22.0, &HashMap::new());
        let ticks = p.sim.run();
        assert!(ticks <= 160, "settled in {ticks} ticks");
        assert!(p.sim.is_settled());
        let before = p.sim.positions();
        p.sim.tick();
        p.sim.tick();
        assert_eq!(before, p.sim.positions(), "no motion after settling");
    }

    #[test]
    fn free_nodes_stay_in_bounds_and_apart() {
        let p = settled(&fixture());
        let free: Vec<&SimNode> = p.sim.nodes[p.pins..].iter().collect();
        for n in &free {
            assert!(
                (FREE_MIN_X..=FREE_MAX_X).contains(&n.x),
                "{} x {}",
                n.id,
                n.x
            );
            assert!(
                (FREE_MIN_Y..=FREE_MAX_Y).contains(&n.y),
                "{} y {}",
                n.id,
                n.y
            );
        }
        let mut worst = f32::MAX;
        for i in 0..free.len() {
            for j in (i + 1)..free.len() {
                let d = dist((free[i].x, free[i].y), (free[j].x, free[j].y));
                worst = worst.min(d / (free[i].radius + free[j].radius));
            }
        }
        // 24 nodes with their rings in a 520 x 440 box: rings may touch, no
        // node may sit on another.
        assert!(worst > 0.55, "closest pair at {worst:.2} of their radii");
    }

    #[test]
    fn synergy_pulls_and_a_counter_that_wins_against_you_pushes() {
        // The same three candidates with and without their relations: each
        // relation moves its own candidate the way it says.
        let base = GraphResult {
            locked: vec![
                pin("Ally", true, 6, Some(Role::Bot)),
                pin("Threat", false, 7, Some(Role::Bot)),
            ],
            candidates: vec![cand("Friend", 70), cand("Loner", 70), cand("Victim", 70)],
            ..GraphResult::default()
        };
        let mut related = base.clone();
        related.edges = vec![
            edge("Friend", "Ally", RelationKind::Synergy, 3.0),
            edge("Victim", "Threat", RelationKind::Counter, -3.0),
        ];
        let at = |g: &GraphResult, id: &str| {
            let p = settled(g);
            let n = p.sim.nodes.iter().find(|n| n.id == id).unwrap();
            (n.x, n.y)
        };
        let ally = (PIN_LEFT_X, row_y(3));
        let threat = (PIN_RIGHT_X, row_y(3));
        assert!(dist(at(&related, "Friend"), ally) < dist(at(&base, "Friend"), ally) - 20.0);
        assert!(dist(at(&related, "Victim"), threat) > dist(at(&base, "Victim"), threat) + 20.0);
    }

    #[test]
    fn higher_fit_sits_nearer_the_middle() {
        let g = GraphResult {
            candidates: (0..10)
                .map(|i| cand(&format!("C{i}"), 95 - i * 6))
                .collect(),
            ..GraphResult::default()
        };
        let p = settled(&g);
        let c = |i: usize| {
            let n = &p.sim.nodes[p.free_index(i)];
            dist((n.x, n.y), CENTRE)
        };
        let top: f32 = (0..3).map(c).sum::<f32>() / 3.0;
        let low: f32 = (7..10).map(c).sum::<f32>() / 3.0;
        assert!(
            top < low,
            "top three {top:.0} from the centre, bottom three {low:.0}"
        );
    }

    #[test]
    fn a_dragged_node_holds_and_the_rest_settles_again() {
        let mut p = settled(&fixture());
        let i = p.free_index(3);
        p.sim.fix(i, 300.0, 120.0);
        p.sim.reheat(0.3);
        assert!(!p.sim.is_settled());
        for _ in 0..30 {
            p.sim.tick();
        }
        assert_eq!((p.sim.nodes[i].x, p.sim.nodes[i].y), (300.0, 120.0));
        p.sim.release(i);
        p.sim.run();
        assert!(p.sim.is_settled());
        // Pins never move, even when "dragged".
        p.sim.fix(0, 400.0, 400.0);
        assert_eq!(p.sim.nodes[0].x, PIN_LEFT_X);
    }

    #[test]
    fn a_warm_start_keeps_known_positions_and_cools_faster() {
        let first = settled(&fixture());
        let prev: HashMap<String, (f32, f32)> = first
            .sim
            .nodes
            .iter()
            .map(|n| (n.id.clone(), (n.x, n.y)))
            .collect();
        let again = plan(&fixture(), 24, radius, 22.0, &prev);
        assert_eq!(again.sim.alpha, 0.5);
        let n = &again.sim.nodes[again.free_index(0)];
        assert_eq!((n.x, n.y), prev[&n.id]);
    }

    #[test]
    fn edges_map_to_forces() {
        let syn = link_force(&edge("a", "b", RelationKind::Synergy, 2.0)).unwrap();
        assert!(syn.attract);
        let beats = link_force(&edge("a", "b", RelationKind::Counter, 2.0)).unwrap();
        assert!(beats.attract && beats.k < syn.k && beats.rest > syn.rest);
        let beaten = link_force(&edge("a", "b", RelationKind::Counter, -2.0)).unwrap();
        assert!(!beaten.attract);
        let against = link_force(&edge("a", "b", RelationKind::Synergy, -1.0)).unwrap();
        assert!(!against.attract);
        assert!(link_force(&edge("a", "b", RelationKind::Synergy, 0.0)).is_none());
    }

    #[test]
    fn only_drawn_candidates_and_known_picks_get_edges() {
        let mut g = fixture();
        g.edges
            .push(edge("C23", "Nobody", RelationKind::Synergy, 2.0));
        let p = plan(&g, 12, radius, 22.0, &HashMap::new());
        assert_eq!(p.sim.nodes.len(), 8 + 12);
        assert!(p
            .edges
            .iter()
            .all(|e| e.free >= p.pins && e.pinned < p.pins));
        assert!(p.edges.iter().all(|e| e.free < p.pins + 12));
    }

    /// The budget in DESIGN.md 6.8: each tick well under 8 ms. Natively in
    /// a debug build this is a few hundred microseconds; the browser
    /// numbers are measured on the page (`data-sim-*`) and in the e2e.
    #[test]
    fn tick_cost_is_far_inside_the_budget() {
        let mut p = plan(&fixture(), 24, radius, 22.0, &HashMap::new());
        let start = std::time::Instant::now();
        let ticks = p.sim.run();
        let total = start.elapsed();
        let per = total.as_secs_f64() * 1000.0 / ticks.max(1) as f64;
        println!(
            "graph layout: {} nodes, {} links, {ticks} ticks, {:.3} ms per tick, {:.2} ms in all",
            p.sim.nodes.len(),
            p.sim.links.len(),
            per,
            total.as_secs_f64() * 1000.0
        );
        assert!(per < 8.0, "{per:.3} ms per tick");
    }
}
