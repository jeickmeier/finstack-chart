//! FIX-11: independent bounded-executor schedules, stale completions and exact source reuse.
use chart_core::{scheduling::*, *};
use std::sync::Arc;
fn key() -> CompatibilityStamp {
    CompatibilityStamp {
        epoch: SourceEpoch::new(1),
        definition: Revision::new(1),
        viewport: Revision::new(1),
        layout: Revision::new(1),
        resources: Revision::new(1),
        presentation: Revision::new(1),
    }
}
#[test]
fn continuous_arrivals_admit_progress_without_canceling_active_work() {
    let mut q = PreparationScheduler::new();
    let mut presented = vec![];
    q.submit(key(), Revision::new(1), 1).unwrap();
    let mut active = q.start().unwrap();
    for arrival in 2..=100 {
        q.submit(key(), Revision::new(arrival), arrival).unwrap();
        assert!(q.start().is_none());
        assert_eq!(q.metrics().active, 1);
        assert_eq!(q.metrics().pending, 1);
        if arrival % 10 == 0 {
            // A deliberately slow build is behind newest accepted data and must still present.
            assert_eq!(q.complete(active.token, true), CompletionOutcome::Ready);
            assert!(q.present(active.token));
            presented.push(active.input);
            active = q.start().unwrap();
        }
    }
    assert_eq!(q.complete(active.token, true), CompletionOutcome::Ready);
    assert!(q.present(active.token));
    presented.push(active.input);
    assert_eq!(presented, vec![1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    assert_eq!(q.metrics().coalesced, 89);
    assert_eq!(q.metrics().lag, Some(0));
    assert_eq!(q.metrics().active, 0);
    assert_eq!(q.metrics().pending, 0);
}
#[test]
fn incompatible_spec_viewport_layout_resource_palette_and_epoch_completions_are_rejected() {
    for field in 0..6 {
        let mut q = PreparationScheduler::new();
        q.submit(key(), Revision::new(5), ()).unwrap();
        let old = q.start().unwrap();
        let mut next = key();
        match field {
            0 => next.definition = Revision::new(2),
            1 => next.viewport = Revision::new(2),
            2 => next.layout = Revision::new(2),
            3 => next.resources = Revision::new(2),
            4 => next.presentation = Revision::new(2),
            _ => next.epoch = SourceEpoch::new(2),
        };
        q.submit(next, Revision::new(if field == 5 { 0 } else { 6 }), ())
            .unwrap();
        assert_eq!(q.complete(old.token, true), CompletionOutcome::Stale);
        assert!(!q.present(old.token));
        let new = q.start().unwrap();
        assert_eq!(q.complete(old.token, true), CompletionOutcome::Stale);
        assert_eq!(q.metrics().active, 1);
        assert_eq!(q.complete(new.token, true), CompletionOutcome::Ready);
        assert!(q.present(new.token));
        assert!(!q.present(old.token));
        // Even a completion with a manufactured newer store revision cannot consume a slot.
        let mut forged = new.token;
        forged.store = Revision::new(999);
        assert_eq!(q.complete(forged, true), CompletionOutcome::Stale);
    }
}
#[test]
fn failures_late_paints_disposal_and_pending_ownership_are_explicit() {
    let mut q = PreparationScheduler::new();
    let old = Arc::new(1);
    let weak = Arc::downgrade(&old);
    q.submit(key(), Revision::new(1), old).unwrap();
    q.submit(key(), Revision::new(2), Arc::new(2)).unwrap();
    assert!(weak.upgrade().is_none());
    let first = q.start().unwrap();
    q.submit(key(), Revision::new(3), Arc::new(3)).unwrap();
    assert_eq!(q.complete(first.token, false), CompletionOutcome::Failed);
    assert!(!q.present(first.token));
    let second = q.start().unwrap();
    assert_eq!(q.complete(second.token, true), CompletionOutcome::Ready);
    q.submit(key(), Revision::new(4), Arc::new(4)).unwrap();
    let third = q.start().unwrap();
    assert_eq!(q.complete(third.token, true), CompletionOutcome::Ready);
    assert!(!q.present(second.token));
    assert!(q.present(third.token));
    assert!(q.submit(key(), Revision::new(3), Arc::new(3)).is_err());
    q.invalidate();
    assert!(q.submit(key(), Revision::new(3), Arc::new(3)).is_err());
    q.submit(key(), Revision::new(5), Arc::new(5)).unwrap();
    let running = q.start().unwrap();
    let weak = Arc::downgrade(&running.input);
    let pending = Arc::new(6);
    let pending_weak = Arc::downgrade(&pending);
    q.submit(key(), Revision::new(6), pending).unwrap();
    q.dispose();
    assert!(pending_weak.upgrade().is_none());
    assert!(weak.upgrade().is_some());
    assert!(q.start().is_none());
    assert!(q.submit(key(), Revision::new(7), Arc::new(7)).is_err());
    assert_eq!(q.complete(running.token, true), CompletionOutcome::Stale);
    drop(running);
    assert!(weak.upgrade().is_none());
    assert_eq!(q.metrics().active, 0);
}
#[test]
fn independent_dashboard_workers_make_fair_progress_with_one_expensive_chart() {
    // A deterministic round-robin host executor: one expensive chart takes 19 ticks;
    // three ordinary charts take 2/3/5. Arrival is every tick, including while active.
    let costs = [19, 2, 3, 5];
    let mut queues: Vec<_> = (0..4).map(|_| PreparationScheduler::new()).collect();
    let mut jobs = vec![None; 4];
    let mut seen = [0; 4];
    for tick in 1..=200u64 {
        for chart in 0..4 {
            let q = &mut queues[chart];
            q.submit(key(), Revision::new(tick), tick).unwrap();
            if let Some((token, ready_at)) = jobs[chart]
                && tick >= ready_at
            {
                assert_eq!(q.complete(token, true), CompletionOutcome::Ready);
                assert!(q.present(token));
                seen[chart] += 1;
                jobs[chart] = None;
            }
            if jobs[chart].is_none() {
                let job = q.start().unwrap();
                jobs[chart] = Some((job.token, tick + costs[chart]));
            }
            assert!(q.metrics().active <= 1 && q.metrics().pending <= 1);
        }
    }
    assert_eq!(seen, [10, 99, 66, 39]);
    for q in &queues {
        assert!(q.metrics().presented.unwrap().get() > 170);
    }
}
