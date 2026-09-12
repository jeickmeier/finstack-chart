"""AXIS-06 typed immutable transition owner and displayed publication capture."""
import finstack_chart as c

def capture(target: c.FigureSnapshot, previous: c.FigureSnapshot, chart: c.Chart, output: c.Output) -> c.FigureSnapshot:
    plan: c.FigureTransition = target.guide_transition(previous)
    sample = plan.sample(0.5)
    guides: list[dict[str, object]] = sample.presentation()
    assert guides
    chart.acknowledge_frame(sample)
    request = chart.request(output, c.export_options(900., 300.).basis("displayed"))
    return request.prepare()
