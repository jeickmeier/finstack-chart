//! Axis join and sampling state. Schedulers and clocks remain in adapters.
//! Join order follows d3-selection 3.0.0 (ISC; fixtures/axes/licenses/d3-selection).
use super::AxisSide;
use crate::{
    ChartResult, DiagnosticCode, Limits,
    composition::ScaleValue,
    interpolate::{ScalarInterpolator, TransformInterpolator, TransformSyntax},
};
use std::collections::BTreeMap;

const EPSILON: f64 = 1e-6;

/// Compact range-derived axis path, in guide-local coordinates.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideDomain {
    /// Horizontal guides use M x,y V y H x [V y]; vertical guides swap coordinates.
    pub horizontal: bool,
    /// Whether the path has end caps (five numeric tokens instead of three).
    pub caps: bool,
    /// Tokens in compact SVG command order; no arbitrary path grammar is accepted.
    pub numbers: Vec<f64>,
}
impl GuideDomain {
    fn validate(&self) -> ChartResult<()> {
        if self.numbers.len() != if self.caps { 5 } else { 3 }
            || self.numbers.iter().any(|n| !n.is_finite())
        {
            return Err(invalid(
                "Guide domain tokens must be finite and match its topology.",
            ));
        }
        Ok(())
    }
    /// Exact compact path topology used by the D3 axis reference.
    pub fn commands(&self) -> ChartResult<Vec<crate::scene::PathCommand>> {
        use crate::{
            Point,
            scene::PathCommand::{LineTo, MoveTo},
        };
        self.validate()?;
        let n = &self.numbers;
        Ok(match (self.horizontal, self.caps) {
            (true, true) => vec![
                MoveTo(Point::new(n[0], n[1])?),
                LineTo(Point::new(n[0], n[2])?),
                LineTo(Point::new(n[3], n[2])?),
                LineTo(Point::new(n[3], n[4])?),
            ],
            (true, false) => vec![
                MoveTo(Point::new(n[0], n[1])?),
                LineTo(Point::new(n[2], n[1])?),
            ],
            (false, true) => vec![
                MoveTo(Point::new(n[0], n[1])?),
                LineTo(Point::new(n[2], n[1])?),
                LineTo(Point::new(n[2], n[3])?),
                LineTo(Point::new(n[4], n[3])?),
            ],
            (false, false) => vec![
                MoveTo(Point::new(n[0], n[1])?),
                LineTo(Point::new(n[0], n[2])?),
            ],
        })
    }
}
/// One displayed tick group, including exiting groups during a transition.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideTransitionTick {
    /// Stable node identity within this guide's retained lifecycle.
    pub identity: usize,
    /// Semantic value; never reconstructed from a painted position.
    pub value: ScaleValue,
    /// Index in the selected list which supplied this label.
    pub index: usize,
    /// Selected logical label; updates replace it when the transition starts.
    pub label: String,
    /// Guide-local coordinate including the current pixel offset.
    pub position: f64,
    /// Continuous opacity, preserving the D3 enter/exit epsilon.
    pub opacity: f64,
    /// Signed tick-line endpoint in the orthogonal direction.
    pub line_end: f64,
    /// Signed label anchor distance in the orthogonal direction (before dy/typography).
    pub label_offset: f64,
}
/// Coherent geometry/labels from one explicit transition sample.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideTransitionFrame {
    /// Replacement boundary: changing side replaces this entire guide immediately.
    pub side: AxisSide,
    /// Guide translation, independently sampled from local geometry.
    pub translation: [f64; 2],
    /// Current compact path, including intermediate cap topology.
    pub domain: GuideDomain,
    /// Paint order includes exits until completion.
    pub ticks: Vec<GuideTransitionTick>,
    /// Next unused identity, retained through empty selections and interruptions.
    pub next_identity: usize,
}
impl GuideTransitionFrame {
    fn validate(&self, limits: Limits) -> ChartResult<()> {
        self.domain.validate()?;
        crate::limits::require_within(
            self.ticks.len() <= limits.max_items,
            "guide transition tick",
        )?;
        if self.domain.horizontal != self.side.horizontal()
            || self.translation.iter().any(|n| !n.is_finite())
        {
            return Err(invalid(
                "Guide frame orientation and finite translation are required.",
            ));
        }
        let mut identities = std::collections::BTreeSet::new();
        let mut bytes = limits.max_text_bytes;
        for tick in &self.ticks {
            let n = tick.label.len().saturating_add(match &tick.value {
                ScaleValue::Category(c) => c.len(),
                _ => 0,
            });
            crate::limits::require_within(n <= bytes, "guide transition text")?;
            bytes -= n;
            if !identities.insert(tick.identity)
                || tick.identity >= self.next_identity
                || [
                    tick.position,
                    tick.opacity,
                    tick.line_end,
                    tick.label_offset,
                ]
                .iter()
                .any(|n| !n.is_finite())
                || !(0. ..=1.).contains(&tick.opacity)
                || matches!(tick.value, ScaleValue::Number(n) if !n.is_finite())
            {
                return Err(invalid("Guide transition tick metadata is invalid."));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
struct Tween {
    from: GuideTransitionTick,
    to: GuideTransitionTick,
    exit: bool,
}
/// Pure bounded D3 axis join plan, sampled by an explicit normalized host time.
///
/// The prior frame may be an interrupted sample. The prior-position callback must
/// use the previous target scale (D3's retained __axis), not an inverse of that sample.
/// The current callbacks use the new target scale; raw keys and centered positions
/// are separate so band/provider joins do not confuse mapping with decoration.
#[derive(Clone, Debug)]
pub struct GuideTransitionPlan {
    from: GuideTransitionFrame,
    target: GuideTransitionFrame,
    tweens: Vec<Tween>,
    replacement: bool,
    limits: Limits,
    translation: TransformInterpolator,
}
impl GuideTransitionPlan {
    /// Compile a join in O(n log n) time and O(n) storage; no per-tick host objects.
    /// Missing raw scale positions all share D3's undefined key. Missing exit
    /// positions retain their displayed transform; missing enter positions use the target.
    pub fn new(
        from: GuideTransitionFrame,
        mut target: GuideTransitionFrame,
        prior_position: impl Fn(&ScaleValue) -> ChartResult<Option<f64>>,
        current_key: impl Fn(&ScaleValue) -> ChartResult<Option<f64>>,
        current_position: impl Fn(&ScaleValue) -> ChartResult<Option<f64>>,
        limits: Limits,
    ) -> ChartResult<Self> {
        from.validate(limits)?;
        target.validate(limits)?;
        crate::limits::require_within(
            from.ticks.len().saturating_add(target.ticks.len()) <= limits.max_items,
            "guide transition union",
        )?;
        let replacement = from.side != target.side;
        let matrix = |p: [f64; 2]| crate::path::Affine::new([1., 0., 0., 1., p[0], p[1]]);
        let translation = TransformInterpolator::new(
            matrix(from.translation)?,
            matrix(target.translation)?,
            TransformSyntax::Svg,
        )?;
        let mut next_identity = from.next_identity;
        if replacement {
            for tick in &mut target.ticks {
                tick.identity = allocate(&mut next_identity)?;
            }
            target.next_identity = next_identity;
            return Ok(Self {
                from,
                target,
                tweens: vec![],
                replacement,
                limits,
                translation,
            });
        }
        let key = |value: &ScaleValue| -> ChartResult<Option<u64>> {
            Ok(current_key(value)?
                .filter(|n| n.is_finite())
                .map(|n| if n == 0. { 0 } else { n.to_bits() }))
        };
        let mut old = BTreeMap::new();
        for (i, tick) in from.ticks.iter().enumerate() {
            old.entry(key(&tick.value)?).or_insert(i);
        }
        let mut matched = vec![false; from.ticks.len()];
        let mut updates = Vec::with_capacity(target.ticks.len());
        let mut tweens: Vec<_> = from
            .ticks
            .iter()
            .map(|tick| Tween {
                from: tick.clone(),
                to: tick.clone(),
                exit: true,
            })
            .collect();
        for tick in &mut target.ticks {
            if let Some(i) = old.remove(&key(&tick.value)?) {
                matched[i] = true;
                tick.identity = from.ticks[i].identity;
                let mut start = from.ticks[i].clone();
                start.value = tick.value.clone();
                start.index = tick.index;
                start.label.clone_from(&tick.label);
                tweens[i] = Tween {
                    from: start,
                    to: tick.clone(),
                    exit: false,
                };
                updates.push(i);
            } else {
                tick.identity = allocate(&mut next_identity)?;
                let mut start = tick.clone();
                start.opacity = EPSILON;
                start.position = prior_position(&tick.value)?
                    .filter(|n| n.is_finite())
                    .unwrap_or(tick.position);
                updates.push(tweens.len());
                tweens.push(Tween {
                    from: start,
                    to: tick.clone(),
                    exit: false,
                });
            }
        }
        for (i, used) in matched.into_iter().enumerate() {
            if !used {
                tweens[i].to.opacity = EPSILON;
                tweens[i].to.position = current_position(&tweens[i].to.value)?
                    .filter(|n| n.is_finite())
                    .unwrap_or(tweens[i].from.position);
            }
        }
        // Reproduce selection.order, then EnterNode._next insertion, with a linked
        // index list. The minimum original update index is the stationary suffix anchor.
        let count = tweens.len();
        let sentinel = count;
        let mut previous = vec![sentinel; count + 1];
        let mut next = previous.clone();
        let old_count = from.ticks.len();
        for i in 0..old_count {
            insert_before(i, sentinel, &mut previous, &mut next);
        }
        let mut anchor = old_count;
        let mut following = sentinel;
        for &i in updates.iter().rev().filter(|&&i| i < old_count) {
            if i > anchor {
                detach(i, &mut previous, &mut next);
                insert_before(i, following, &mut previous, &mut next);
            }
            anchor = anchor.min(i);
            following = i;
        }
        let mut following = sentinel;
        let mut successors = vec![sentinel; updates.len()];
        for (j, &i) in updates.iter().enumerate().rev() {
            successors[j] = following;
            if i < old_count {
                following = i;
            }
        }
        for (j, &i) in updates.iter().enumerate() {
            if i >= old_count {
                insert_before(i, successors[j], &mut previous, &mut next);
            }
        }
        let mut ordered = Vec::with_capacity(count);
        let mut i = next[sentinel];
        while i != sentinel {
            ordered.push(tweens[i].clone());
            i = next[i];
        }
        target.next_identity = next_identity;
        Ok(Self {
            from,
            target,
            tweens: ordered,
            replacement,
            limits,
            translation,
        })
    }
    /// Sample start, intermediate or final state. End returns the exact target;
    /// reduced motion is implemented by the host requesting 1 immediately.
    pub fn sample(&self, fraction: f64) -> ChartResult<GuideTransitionFrame> {
        if !fraction.is_finite() || !(0. ..=1.).contains(&fraction) {
            return Err(invalid(
                "Guide transition fraction must be finite in [0,1].",
            ));
        }
        if fraction == 1. || self.replacement {
            return Ok(self.target.clone());
        }
        let number = |a, b| ScalarInterpolator::number(a, b).sample(fraction);
        let mut frame = self.target.clone();
        frame.translation = self.translation.components(fraction)?.translate;
        // D3 string interpolation pairs numbers by token ordinal, using the new
        // command pattern even at t=0. Unpaired target numbers are constants.
        for (i, n) in frame.domain.numbers.iter_mut().enumerate() {
            if let Some(&a) = self.from.domain.numbers.get(i) {
                *n = number(a, *n)?;
            }
        }
        frame.ticks = self
            .tweens
            .iter()
            .map(|t| {
                let mut tick = t.to.clone();
                tick.position = number(t.from.position, t.to.position)?;
                tick.opacity = number(t.from.opacity, t.to.opacity)?;
                tick.line_end = number(t.from.line_end, t.to.line_end)?;
                tick.label_offset = number(t.from.label_offset, t.to.label_offset)?;
                Ok(tick)
            })
            .collect::<ChartResult<_>>()?;
        frame.validate(self.limits)?;
        Ok(frame)
    }
    /// Retained identity for each displayed group and whether it exits at completion.
    pub fn identities(&self) -> impl Iterator<Item = (usize, bool)> + '_ {
        self.tweens.iter().map(|t| (t.to.identity, t.exit))
    }
}
fn allocate(next: &mut usize) -> ChartResult<usize> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or_else(|| invalid("Guide transition identity exhausted."))?;
    Ok(id)
}
fn detach(i: usize, previous: &mut [usize], next: &mut [usize]) {
    next[previous[i]] = next[i];
    previous[next[i]] = previous[i];
}
fn insert_before(i: usize, at: usize, previous: &mut [usize], next: &mut [usize]) {
    let before = previous[at];
    previous[i] = before;
    next[i] = at;
    next[before] = i;
    previous[at] = i;
}
fn invalid(message: &str) -> crate::Diagnostic {
    crate::scales::error(DiagnosticCode::Validation, message)
}
