// Deterministic front-chain packing and minimum enclosure adapted from D3 (ISC, LICENSE).
use super::*;
/// Standalone circle geometry for packing and enclosure helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circle {
    /// Center x coordinate.
    pub x: f64,
    /// Center y coordinate.
    pub y: f64,
    /// Nonnegative radius in layout units.
    pub r: f64,
}
impl Circle {
    fn validate(self) -> ChartResult<()> {
        NodeGeometry::Circle {
            x: self.x,
            y: self.y,
            r: self.r,
        }
        .validate()
    }
    fn finite(self) -> ChartResult<()> {
        if [self.x, self.y, self.r].iter().all(|v| v.is_finite()) {
            Ok(())
        } else {
            Err(numerical("Circle packing arithmetic overflowed."))
        }
    }
}
/// Leaf radius policy; native/registered accessors use Explicit at invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackRadius {
    /// Square-root leaf values, then fit to the smaller output extent.
    Fitted,
    /// Exact unscaled leaf radii supplied by the accessor.
    Explicit,
}
/// Hierarchical circle packing controls.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PackOptions {
    /// Destination width and height; explicit radii are not rescaled to fit.
    pub size: [f64; 2],
    /// Fitted/default or explicit radius accessor mode.
    pub radius: PackRadius,
    /// Finite signed padding, or the default supplied to a custom accessor.
    pub padding: f64,
}
impl Default for PackOptions {
    fn default() -> Self {
        Self {
            size: [1., 1.],
            radius: PackRadius::Fitted,
            padding: 0.,
        }
    }
}
/// Typed native packing accessors, also implemented by registered portable operations.
pub trait PackAccessors {
    /// Exact leaf radius. Explicit mode requires a supplied accessor.
    fn radius(&mut self, _node: NodeView<'_>) -> ChartResult<f64> {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Explicit packing radius accessor was not supplied.",
        ))
    }
    /// Node padding; the default returns its configured constant.
    fn padding(&mut self, _node: NodeView<'_>, configured: f64) -> ChartResult<f64> {
        Ok(configured)
    }
}
impl PackAccessors for () {}
impl Hierarchy {
    /// Hierarchical fitted packing with the reference's deterministic random sequence.
    pub fn pack(&self, options: PackOptions) -> ChartResult<HierarchyLayout> {
        self.pack_with(options, &mut ())
    }
    /// Hierarchical packing through typed native/registered radius and padding accessors.
    pub fn pack_with(
        &self,
        options: PackOptions,
        accessors: &mut impl PackAccessors,
    ) -> ChartResult<HierarchyLayout> {
        super::layout::extent(options.size)?;
        if !options.padding.is_finite() {
            return Err(numerical("Packing padding must be finite."));
        }
        if options.radius == PackRadius::Fitted {
            self.require_values()?;
        }
        let mut work = Work::new(self.limits);
        work.charge(self.len())?;
        let mut random = Lcg(1);
        let mut circles = vec![Circle::default(); self.len()];
        circles[self.root].x = options.size[0] / 2.;
        circles[self.root].y = options.size[1] / 2.;
        for i in self.order(VisitOrder::PreOrder)? {
            work.charge(1)?;
            if self.nodes[i].children.is_empty() {
                let r = match options.radius {
                    PackRadius::Fitted => self.nodes[i].value.expect("checked aggregation").sqrt(),
                    PackRadius::Explicit => accessors.radius(self.view(i))?,
                };
                circles[i].r = weight(r)?;
            }
        }
        let k = if options.radius == PackRadius::Explicit {
            pack_children(
                self,
                &mut circles,
                0.5,
                Some((accessors, options.padding)),
                &mut random,
                &mut work,
            )?;
            1.
        } else {
            pack_children(
                self,
                &mut circles,
                1.,
                None::<(&mut (), f64)>,
                &mut random,
                &mut work,
            )?;
            let min = options.size[0].min(options.size[1]);
            let padding_scale = circles[self.root].r / min;
            pack_children(
                self,
                &mut circles,
                padding_scale,
                Some((accessors, options.padding)),
                &mut random,
                &mut work,
            )?;
            // The checked profile collapses all-zero fitted trees instead of emitting NaN.
            if circles[self.root].r == 0. {
                0.
            } else {
                min / (2. * circles[self.root].r)
            }
        };
        for i in self.order(VisitOrder::PreOrder)? {
            work.charge(1)?;
            circles[i].r *= k;
            if let Some(parent) = self.nodes[i].parent {
                circles[i].x = circles[parent].x + k * circles[i].x;
                circles[i].y = circles[parent].y + k * circles[i].y;
            }
        }
        HierarchyLayout::new(
            self.clone(),
            circles
                .into_iter()
                .map(|c| NodeGeometry::Circle {
                    x: c.x,
                    y: c.y,
                    r: c.r,
                })
                .collect(),
        )
    }
}
fn pack_children<A: PackAccessors>(
    tree: &Hierarchy,
    circles: &mut [Circle],
    k: f64,
    mut padding: Option<(&mut A, f64)>,
    random: &mut Lcg,
    work: &mut Work,
) -> ChartResult<()> {
    for i in tree.order(VisitOrder::PostOrder)? {
        work.charge(1)?;
        let children = &tree.nodes[i].children;
        if children.is_empty() {
            continue;
        }
        let value = if let Some((accessors, configured)) = &mut padding {
            let value = accessors.padding(tree.view(i), *configured)?;
            if !value.is_finite() {
                return Err(numerical(
                    "Packing padding accessor must return finite values.",
                ));
            }
            value
        } else {
            0.
        };
        let raw = value * k;
        let r = if raw.is_nan() { 0. } else { raw };
        if !r.is_finite() {
            return Err(numerical("Packing padding multiplication overflowed."));
        }
        let mut local = children
            .iter()
            .map(|&c| Circle {
                r: circles[c].r + r,
                ..circles[c]
            })
            .collect::<Vec<_>>();
        let enclosure = pack_siblings_random(&mut local, random, work)?;
        for (&c, mut geometry) in children.iter().zip(local) {
            geometry.r -= r;
            circles[c] = geometry;
        }
        circles[i].r = enclosure + r;
        circles[i].finite()?;
    }
    Ok(())
}
/// Place independent circles with exact supplied radii, returning an owned ordered result.
pub fn pack_siblings(circles: &[Circle], limits: HierarchyLimits) -> ChartResult<Vec<Circle>> {
    within(
        circles.len() <= limits.max_nodes,
        "Packing circle budget exceeded.",
    )?;
    for c in circles {
        c.validate()?;
    }
    let mut output = circles.to_vec();
    pack_siblings_random(&mut output, &mut Lcg(1), &mut Work::new(limits))?;
    for c in &output {
        c.validate()?;
    }
    Ok(output)
}
/// Deterministic minimum enclosing circle, or None for an empty input.
pub fn pack_enclose(circles: &[Circle], limits: HierarchyLimits) -> ChartResult<Option<Circle>> {
    within(
        circles.len() <= limits.max_nodes,
        "Enclosure circle budget exceeded.",
    )?;
    for c in circles {
        c.validate()?;
    }
    let output = enclose_random(circles, &mut Lcg(1), &mut Work::new(limits))?;
    if let Some(c) = output {
        c.validate()?;
    }
    Ok(output)
}
struct Lcg(u32);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(self.0) / 4_294_967_296.
    }
}
fn place(b: Circle, a: Circle, r: f64) -> Circle {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let d2 = dx * dx + dy * dy;
    if d2 != 0. {
        let a2 = (a.r + r) * (a.r + r);
        let b2 = (b.r + r) * (b.r + r);
        if a2 > b2 {
            let x = (d2 + b2 - a2) / (2. * d2);
            let y = (b2 / d2 - x * x).max(0.).sqrt();
            Circle {
                x: b.x - x * dx - y * dy,
                y: b.y - x * dy + y * dx,
                r,
            }
        } else {
            let x = (d2 + a2 - b2) / (2. * d2);
            let y = (a2 / d2 - x * x).max(0.).sqrt();
            Circle {
                x: a.x + x * dx - y * dy,
                y: a.y + x * dy + y * dx,
                r,
            }
        }
    } else {
        Circle {
            x: a.x + r,
            y: a.y,
            r,
        }
    }
}
fn intersects(a: Circle, b: Circle) -> bool {
    let dr = a.r + b.r - 1e-6;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dr > 0. && dr * dr > dx * dx + dy * dy
}
fn score(a: Circle, b: Circle) -> f64 {
    let ab = a.r + b.r;
    let dx = (a.x * b.r + b.x * a.r) / ab;
    let dy = (a.y * b.r + b.y * a.r) / ab;
    dx * dx + dy * dy
}
fn pack_siblings_random(
    circles: &mut [Circle],
    random: &mut Lcg,
    work: &mut Work,
) -> ChartResult<f64> {
    let n = circles.len();
    work.charge(n)?;
    if n == 0 {
        return Ok(0.);
    }
    circles[0].x = 0.;
    circles[0].y = 0.;
    if n == 1 {
        return Ok(circles[0].r);
    }
    circles[0].x = -circles[1].r;
    circles[1].x = circles[0].r;
    circles[1].y = 0.;
    if n == 2 {
        return Ok(circles[0].r + circles[1].r);
    }
    circles[2] = place(circles[1], circles[0], circles[2].r);
    circles[2].finite()?;
    let mut next = vec![0; n];
    let mut previous = vec![0; n];
    next[0] = 1;
    next[1] = 2;
    next[2] = 0;
    previous[0] = 2;
    previous[1] = 0;
    previous[2] = 1;
    let (mut a, mut b, mut i) = (0, 1, 3);
    'pack: while i < n {
        work.charge(1)?;
        circles[i] = place(circles[a], circles[b], circles[i].r);
        circles[i].finite()?;
        let (mut j, mut k) = (next[b], previous[a]);
        let (mut sj, mut sk) = (circles[b].r, circles[a].r);
        loop {
            work.charge(1)?;
            if sj <= sk {
                if intersects(circles[j], circles[i]) {
                    b = j;
                    next[a] = b;
                    previous[b] = a;
                    continue 'pack;
                }
                sj += circles[j].r;
                j = next[j];
            } else {
                if intersects(circles[k], circles[i]) {
                    a = k;
                    next[a] = b;
                    previous[b] = a;
                    continue 'pack;
                }
                sk += circles[k].r;
                k = previous[k];
            }
            if j == next[k] {
                break;
            }
        }
        previous[i] = a;
        next[i] = b;
        next[a] = i;
        previous[b] = i;
        b = i;
        let mut aa = score(circles[a], circles[next[a]]);
        let mut c = next[i];
        while c != b {
            work.charge(1)?;
            let ca = score(circles[c], circles[next[c]]);
            if ca < aa {
                a = c;
                aa = ca;
            }
            c = next[c];
        }
        b = next[a];
        i += 1;
    }
    let mut front = vec![circles[b]];
    let mut c = next[b];
    while c != b {
        work.charge(1)?;
        front.push(circles[c]);
        c = next[c];
    }
    let enclosing = enclose_random(&front, random, work)?.expect("nonempty chain");
    for c in circles {
        c.x -= enclosing.x;
        c.y -= enclosing.y;
        c.finite()?;
    }
    Ok(enclosing.r)
}
fn encloses_not(a: Circle, b: Circle) -> bool {
    let dr = a.r - b.r;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dr < 0. || dr * dr < dx * dx + dy * dy
}
fn encloses_weak(a: Circle, b: Circle) -> bool {
    let dr = a.r - b.r + a.r.max(b.r).max(1.) * 1e-9;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dr > 0. && dr * dr > dx * dx + dy * dy
}
fn encloses_all(a: Circle, basis: &[Circle]) -> bool {
    basis.iter().all(|&b| encloses_weak(a, b))
}
fn enclose_random(
    circles: &[Circle],
    random: &mut Lcg,
    work: &mut Work,
) -> ChartResult<Option<Circle>> {
    let mut shuffled = circles.to_vec();
    let mut m = shuffled.len();
    while m > 0 {
        work.charge(1)?;
        let i = (random.next() * m as f64) as usize;
        m -= 1;
        shuffled.swap(i, m);
    }
    let mut basis = Vec::new();
    let mut enclosing = None;
    let mut i = 0;
    while i < shuffled.len() {
        work.charge(1)?;
        let p = shuffled[i];
        if enclosing.is_some_and(|e| encloses_weak(e, p)) {
            i += 1;
        } else {
            basis = extend_basis(&basis, p, work)?;
            let e = match basis.as_slice() {
                [a] => *a,
                [a, b] => basis2(*a, *b),
                [a, b, c] => basis3(*a, *b, *c),
                _ => return Err(invalid("Invalid enclosure basis.")),
            };
            e.finite()?;
            enclosing = Some(e);
            i = 0;
        }
    }
    Ok(enclosing)
}
fn extend_basis(basis: &[Circle], p: Circle, work: &mut Work) -> ChartResult<Vec<Circle>> {
    if encloses_all(p, basis) {
        return Ok(vec![p]);
    }
    for &b in basis {
        work.charge(1)?;
        if encloses_not(p, b) && encloses_all(basis2(b, p), basis) {
            return Ok(vec![b, p]);
        }
    }
    for i in 0..basis.len().saturating_sub(1) {
        for j in i + 1..basis.len() {
            work.charge(1)?;
            let (a, b) = (basis[i], basis[j]);
            if encloses_not(basis2(a, b), p)
                && encloses_not(basis2(a, p), b)
                && encloses_not(basis2(b, p), a)
                && encloses_all(basis3(a, b, p), basis)
            {
                return Ok(vec![a, b, p]);
            }
        }
    }
    Err(numerical(
        "Cannot construct a finite circle enclosure basis.",
    ))
}
fn basis2(a: Circle, b: Circle) -> Circle {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dr = b.r - a.r;
    let length = (dx * dx + dy * dy).sqrt();
    Circle {
        x: (a.x + b.x + dx / length * dr) / 2.,
        y: (a.y + b.y + dy / length * dr) / 2.,
        r: (length + a.r + b.r) / 2.,
    }
}
fn basis3(a: Circle, b: Circle, c: Circle) -> Circle {
    let a2 = a.x - b.x;
    let a3 = a.x - c.x;
    let b2 = a.y - b.y;
    let b3 = a.y - c.y;
    let c2 = b.r - a.r;
    let c3 = c.r - a.r;
    let d1 = a.x * a.x + a.y * a.y - a.r * a.r;
    let d2 = d1 - b.x * b.x - b.y * b.y + b.r * b.r;
    let d3 = d1 - c.x * c.x - c.y * c.y + c.r * c.r;
    let ab = a3 * b2 - a2 * b3;
    let xa = (b2 * d3 - b3 * d2) / (ab * 2.) - a.x;
    let xb = (b3 * c2 - b2 * c3) / ab;
    let ya = (a3 * d2 - a2 * d3) / (ab * 2.) - a.y;
    let yb = (a2 * c3 - a3 * c2) / ab;
    let aa = xb * xb + yb * yb - 1.;
    let bb = 2. * (a.r + xa * xb + ya * yb);
    let cc = xa * xa + ya * ya - a.r * a.r;
    let r = -(if aa.abs() > 1e-6 {
        (bb + (bb * bb - 4. * aa * cc).sqrt()) / (2. * aa)
    } else {
        cc / bb
    });
    Circle {
        x: a.x + xa + xb * r,
        y: a.y + ya + yb * r,
        r,
    }
}
