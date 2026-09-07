# Linked views, editing and host controls

WP-17 extends the [state/action contract](state-action-contract.md) and
[interaction contract](interaction-contract.md). Shared semantics live in chart-core;
native controls and services live in gpui-charts and the application.

## Semantic links

`LinkMessage::from_event` captures effective durable state with an origin and revision.
It suppresses linked-origin echoes. `resolve` maps named axes and stable semantic
identities to a receiver's own layers/panels, returning a normal synchronization action
and preserved origin. No renderer index or sender layer ID is used as a row identity.
Numeric value spaces and timestamp unit/timezone must match; category endpoints are
labels in the receiver's catalog. Axis mappings explicitly assert application meaning;
there is no implicit currency or unit conversion. Partial windows preserve other axes.

Source identity includes epoch, dataset and key. Aggregate/model targets also preserve
their input revisions and scope. Hidden or clipped prepared targets remain linkable;
hover and keyboard focus still require presentation. Missing targets either reject the
message or return an explicit unmatched list under `ReportAndOmit`. The reducer keeps
bounded origin/revision records, ignores older/repeated updates, and rejects changed
payloads reused under one origin/revision. Linked changes do not mutate source data.

## Annotation producers

`AnnotationEditor` pins the presented chart and original annotation. Pointer previews
use total displacement from that basis through the common projection/inverse APIs.
Constraints can lock either direction, bound/snap numeric or exact timestamp values,
restrict categories, and preserve ordered range endpoints. Timestamp constraint wire
values are decimal strings. Keyboard nudges use the same constraints and direction.
Snapping rounds to the nearer grid point, choosing the greater grid value on a tie,
then clamps. Invalid/crossing ranges reject without changing committed state.

The producer returns annotation values; the reducer owns begin/preview/commit/cancel.
One completed drag is one undo entry. Escape, focus/capture loss and interrupted edits
discard previews. Resize does not change a gesture's original basis. Annotation
`ConnectorOrigin::Anchor` draws an exact anchor-to-callout line independently of label
offset, suitable for thresholds and ranges; the default label connector is preserved.
Native handles have priority over chart pan/selection. Editing observations requires
separate explicit application data transactions; annotation tools cannot write rows.

## Native hooks and accessible data

Applications subscribe to `ChartHostEvent`, explicitly enable handled `HostCommand`s,
and install replaceable toolbar, legend, context-menu and tooltip factories. Requests
retain the actual scene, selection, focus and optional local position. Disabled commands
return a diagnostic. Native control containers stop pointer/scroll propagation so a
button does not also begin chart navigation. Default bindings remain chart-focus scoped
and can be replaced. The example implements clipboard and supplied-font export services.
Full live-overlay export coherence belongs to WP-20.

`Inspector::accessible_page` returns up to 256 ordered visible semantic targets with a
scene stamp, total count, focus/selection flags and exact identity/value descriptions.
Source descriptions include up to 16 fields with bounded text, explicit missing and
non-finite values, units/timestamps and an omitted-field count. Aggregate and model
descriptions report their distinct provenance. Exact 64-bit identities stay strings.
The native view exposes a chart summary, active target and annotation handles through
GPUI accessibility; the example adds an accessible linked data table without retaining
one native entity for every source observation.

On the tested macOS/GPUI target, the accessibility tree exposes summary, table cells,
selection descriptions and annotation values. The automation inspector still reports
the OS-focused element as the window despite chart keyboard focus. VoiceOver speech,
complete screen-reader traversal and Linux accessibility are not certified. See
[executed evidence](evidence/wp-17-completion-2026-09-07.md).
