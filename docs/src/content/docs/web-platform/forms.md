---
title: Forms and input
description: Edit controls, validate values and understand submission and UI limits.
---

Brimp supports script-driven form editing and direct input actions. Values,
selection, validity and events are updated without a visual form control.

| Capability | Support and limits |
| --- | --- |
| Values and state | Text, number, range, date, time, datetime-local, color, checkbox/radio and file state; month/week use Safari’s text fallback |
| Validation | Required, email/URL, pattern, numeric/temporal and user-edit length checks; measured Safari English messages |
| Editing | Selection, range replacement, keyboard editing and cancelable activation; no visual selection or native picker |
| Submission | Existing form submission supports GET; metadata does not add POST or dialog submission |
| Files | FileList state and host-supplied files; no directory picker or filesystem-entry backend |

`checkValidity()` reports validity and emits invalid events. `reportValidity()`
preserves that result but does not focus, scroll or show a validation popup.

## Form ownership and labels

Controls can belong to forms outside their ancestor chain. Association and
label lists follow tree and attribute changes, including supported parser-created
associations. Form named properties that shadow methods remain a known gap.

<details>
<summary>Detailed behavior and edge cases</summary>

Label control lookup and live labels lists use canonical attributes and tree state.
They follow measured Safari behavior for duplicate IDs, hidden controls, shadow
boundaries and form-associated custom definitions. Ordinary detached trees return
empty labels lists; detached shadow trees retain implicit labels. Lists preserve
Safari's ID-change cache behavior and invalidate on element-tree or changed
for/type attributes. htmlFor is a DOMString reflection with lossless UTF-16 storage.
Label click forwarding is a separate behavior. Form ownership includes parser-
assigned non-ancestor forms in malformed table markup. The native DOM preserves
these associations when both nodes move together and resets them when separated,
on form-attribute changes or cloning. Full document parsing and document.write
retain parser state across script/chunk boundaries; fragment parsing does not
create it. Input/button form and label.form are readonly canonical getters.
Shared Safari tests cover 245 parser association cases and 17 radio sequences,
plus moves, adoption, cloning, namespaces, exact IDs and document replacement.
Input datalist association follows Safari's supported types, first-ID lookup and
tree boundaries, including its exclusion of ordinary DocumentFragment roots.
`webkitdirectory` reflects boolean attribute presence. `webkitEntries` is a
stable frozen empty array per input; directory picking and filesystem entry
objects are not implemented. Safari's unsupported month/week types reflect text.

</details>

## Values and reset

Value sanitization and dirty-state rules depend on the input type. Reset restores
defaults and honors cancellation. Clones preserve supported edited state but
omit files, custom errors, selection positions and listeners.

<details>
<summary>Detailed behavior and edge cases</summary>

Input values distinguish attribute-backed button/hidden and checkbox/radio modes
from editable text and file modes. Text, email and URL sanitization, canonical
`defaultValue` reflection and dirty-value transitions follow measured Safari
behavior. Leaving file mode discards selected files while preserving the previous
edited value. Attribute and Attr-node changes use the same transition logic.
Date/time strings enforce Safari's calendar and time limits; datetime-local values
are normalized. Range values clamp and snap using Safari decimal behavior, with
canonical `min`/`max`/`step` reflections. The shared sanitization tests cover 743
measured cases through four mutation paths each. Decimal arithmetic is a private
adaptation of WebKit's BSD-licensed Decimal implementation; it adds no picker or
rendering backend.

Cloning and importing preserve input/textarea dirty values and input checkedness
and indeterminate state, including controls in template contents. Clones start
with a collapsed selection at zero and omit selected files, custom validity and
listeners. Textarea defaults use direct child text; values normalize CR/CRLF.

</details>

## Numbers, dates and stepping

`valueAsNumber`, `valueAsDate`, `stepUp()` and `stepDown()` support the input types
described below. They update values without generating input/change events.
No date/time picker is provided.

<details>
<summary>Detailed behavior and edge cases</summary>

Input valueAsNumber supports number, range, date, time and datetime-local. Date/time
conversion uses UTC components, Safari millisecond rounding and the existing
WebKit decimal helpers. Time serialization reflects step precision; datetime-local
serialization removes redundant zero fields. valueAsDate returns fresh Date objects
for date/time controls and accepts Safari's legacy Date, primitive-number and
primitive-string inputs. Invalid date serialization restores attribute-backed
values; NaN/infinite valueAsNumber assignments throw NotSupportedError without
changing state. Shared tests cover 2,340 Safari conversion, formatting and getter
cases, plus descriptors, coercion, intrinsic methods, cloning/reset and mutations.
Month/week remain Safari's text fallback. No picker or rendering backend is added.
Input stepUp/stepDown support number, range, date, time and datetime-local in core.
They use canonical input state, decimal step alignment and bounds, and Safari's
signed 32-bit count conversion. Temporal steps round to whole days or milliseconds
as appropriate. Stepping updates dirty values without input/change events or
attribute mutations. Shared tests cover 2,500 Safari cases, including decimal
rounding quirks, count overflow, unsupported types, invalid/any steps, default
restoration, calendar rollover and saturation, plus descriptors and clone/reset.

</details>

## Color inputs

Colors are parsed and serialized in core, including sRGB/Display-P3 and alpha.
No graphics plugin is needed. System colors use fixed light-appearance values.

<details>
<summary>Detailed behavior and edge cases</summary>

Color inputs parse named colors, CSS color functions, relative colors and color
mixing into sRGB or Display-P3 values using bundled CSSTools primitives and
Safari float conversion/rounding. The canonical `alpha` and `colorSpace`
reflections select serialization and preserve dirty-value semantics through
attribute changes, type transitions, reset and cloning. Shared tests compare 576
color cases through four paths and ten transition sequences with installed Safari.
System colors use measured Safari light-appearance defaults; they do not track
the host OS theme. Color parsing belongs to core and requires no graphics plugin.

</details>

## Checkboxes and radio buttons

Checkedness, dirty state, radio groups and indeterminate state are retained.
Activation changes state before click listeners and restores it when canceled.

<details>
<summary>Detailed behavior and edge cases</summary>

Input checkedness and defaultChecked use independent dirty state. Radio exclusivity
follows Safari's names, form ownership, tree boundaries, insertion and attribute
changes; cloning retains dirty state and reset restores defaults. Detached ordinary
trees, shadow trees and parsed documents follow measured Safari distinctions.
Checkbox/radio activation runs before click listeners and restores the appropriate
state on cancellation, shared by the input controller and script-dispatched
mouse/pointer clicks. Checkbox activation also clears indeterminate state.
Connected controls emit input/change events with Safari's shadow-boundary behavior.
Shared tests cover 556 recorded state, activation and group sequences, plus
canonical attributes, descriptors, cloning and reset in documents with/without a
browsing context. This does not add browser rendering or radio keyboard navigation.

</details>

## Selection and keyboard editing

Inputs and textareas retain UTF-16 selection and replacement state. Keyboard
edits track a separate visible buffer so incomplete numeric input and user-edit
length validation can be represented.

<details>
<summary>Detailed behavior and edge cases</summary>

Inputs and textareas share canonical UTF-16 selection state and range replacement.
The selection APIs follow measured Safari conversion, type gating, focus and
value/reset/type-transition behavior, including queued select events and coalesced
selectionchange events. The shared tests cover 640 selection/replacement cases.
Safari's case-insensitive setRangeText modes and preserve behavior for unknown
modes are retained. Selection painting and document Selection integration are
separate surfaces.

The input constraint layer reads canonical values and attributes for required
text/date controls, checkbox/file requirements and named radio groups, including
detached groups. It checks email lists item by item, parses absolute URLs, compiles
Unicode-v patterns, and reuses finite decimal stepping for numeric/temporal bounds
and step mismatch, including reversed time ranges. Shared tests cover 3,262
scripted constraint cases and 1,260 radio-group cases. Native text/number edits
retain a private visible buffer and user-edit state, so incomplete numeric text
sets badInput while exposing an empty value, and UTF-16 length constraints apply
to user edits. Script assignments, type changes, reset, cloning and adoption
follow Safari's distinct state rules. Keyboard editing also handles numeric
filtering, maxlength, cancellation and internal selection for email/number
controls without exposing unsupported public selection APIs. Shared tests cover
85 typing sequences, 42 state mutations and 22 continued-typing sequences, plus
events, selection, numeric setter/stepping interactions and Unicode length limits.
These are JavaScript input operations; native date pickers and visual validation
UI are not supplied by this contract.

</details>

## Validation eligibility

Disabled, readonly, type and ancestor state determine whether a control is
validated. Eligibility also affects clicks, selection and form-data inclusion.

<details>
<summary>Detailed behavior and edge cases</summary>

Forms expose association (including controls outside their form), labels, reset,
constraint validity/custom validity, numeric stepping and text selection replacement.
Validation eligibility uses canonical control type, disabled/readonly attributes,
HTML datalist ancestry and Safari's fieldset/first-legend rules. Image inputs are
excluded, and readonly excludes every input type. The nearest disabled fieldset's
first legend exempts its controls even inside an outer disabled fieldset. Private
ancestor-disabled state preserves Safari's mutation behavior: some exemptions
survive fieldset attribute changes and removal, then refresh on insertion. The
shared state also governs script clicks, selection and form-data exclusion.
Shared tests cover 3,360 eligibility cases and 256 mutation sequences, including
shadow boundaries, namespaces, clone/adoption, descriptors and public overrides.
This eligibility contract does not assert complete constraint flags, localized
validation messages or validation UI.

</details>

## Custom errors and validation messages

Custom errors survive reset/adoption but not cloning. Messages follow the
measured Safari English profile; picker-driven updates and other localized UI
are outside the contract.

<details>
<summary>Detailed behavior and edge cases</summary>

Input/button setCustomValidity requires a DOMString argument, preserves UTF-16,
rejects Symbols and failed conversions without changing the previous error, and
emits no input/change/invalid events. Custom errors survive reset and adoption;
cloning/import starts with no custom error. Button ValidityState is stable and
live, with only customError affecting valid. Its message is empty while barred
from validation. Button checkValidity uses canonical private state and dispatches
a trusted, non-bubbling, cancelable invalid event; clearing the error or canceling
that event does not change the already-computed false return. Shared tests cover
595 Safari conversion/event cases plus descriptors, overrides and state lifetime.

Input validationMessage uses the installed Safari English messages, including
error priority, normalized pattern titles, singular length messages and finite
decimal/date/time bound formatting. It is readonly and reads canonical state
without events, mutations or focus changes. Input checkValidity checks eligibility
and dispatches a trusted, cancelable, non-bubbling invalid event; cancellation or
listener mutations do not change its pre-event result. Safari's reversed time
ranges can have a nonempty message while checkValidity succeeds. Clearing a
populated temporal editor can leave its check result invalid until a subsequent
value, constraint, custom-error or eligibility update, although its live flags
and message are already clear. Private state preserves that distinction across
parser initialization, cloning, focus, reset and DOM moves. Shared tests cover
3,712 scripted message/event cases, 85 keyboard sequences and 792 cache transitions.
Rendering-driven picker updates and localized UI outside the measured English
profile are not part of these core JavaScript operations.

</details>

## Report validity without UI

`reportValidity()` preserves the boolean result and trusted invalid events,
including reentrant listener behavior. Its visual presentation is a no-op.

<details>
<summary>Detailed behavior and edge cases</summary>

Input/button reportValidity preserves the core boolean result and trusted invalid
events, including canceled events and reentrant listener mutations. Its visual
presentation is an explicit adapted legacy no-op: no popup, scrolling or focus
change. The callable is non-constructible with Safari descriptors and receiver
checks. Shared tests verify 252 Safari return/event cases; separate runtime tests
verify the deliberate no-UI behavior, which differs from Safari focusing an invalid
connected control.

</details>

## Files and image inputs

File inputs preserve assigned FileList identity and clear it on reset. Image
input dimensions come from attributes, without decoding or CSS layout.

<details>
<summary>Detailed behavior and edge cases</summary>

Input files accepts null/undefined as no-ops and requires a genuinely branded
FileList for other assignments, even when the input is not a file control. File
inputs retain the assigned list identity, including live DataTransfer changes.
Clearing the value or resetting the form empties that shared list; changing away
from file type discards the association, and cloning starts with an empty list.
Canonical file data drives the filename and required validity independently of
public property overrides. Assignment emits no input/change events or mutations.
Input src resolves against the owning document's live base URL and stores a
USVString attribute. Width/height use Safari unsigned conversion and return zero
outside image type. Image dimensions use attributes; CSS layout and intrinsic
image decoding remain explicit no-layout placeholders. These common input
surfaces stay in core, with 78 file-assignment, 308 URL and 360 dimension reference
cases plus shared state-lifetime tests.

</details>

## Attributes and submitter metadata

Input and button attributes use canonical DOM state and Web IDL conversion.
Autocomplete is metadata only. Action/method/encoding overrides do not extend
the existing GET-only form submission transport.

<details>
<summary>Detailed behavior and edge cases</summary>

Input string and boolean reflections use canonical null-namespace attributes,
WebIDL conversions and mutation records. Element receiver checks reject forged
prototype-only objects before entering native DOM operations. Input autocomplete
parses Safari's field, section, address/contact and webauthn tokens, including
hidden-input rules and live type changes. The shared corpus covers 1,068 cases;
it supplies metadata without an autofill or credentials backend. The legacy
`incremental` IDL property remains absent, as in installed Safari.
Input `maxLength`, `minLength` and `size` use canonical integer reflection with
Safari conversion and range errors; length setters check the opposite bound after
conversion. The 624-case corpus includes errors, attribute parsing and type changes.
`formMethod` includes `dialog`; method/encoding overrides retain missing and invalid
states. `formAction` resolves against its owning document and first eligible HTML
base, with 308 measured URL cases plus cloning/import/adoption checks. Created
documents retain Safari's empty internal action URL despite exposing `about:blank`
as `Document.URL`. These metadata contracts do not implement submitter routing,
POST transport or dialog submission; existing form submission supports GET.
Buttons share these form overrides, with a separate 308-case Safari URL reference.
Their name/value/formTarget strings and disabled/formNoValidate booleans preserve
canonical attributes, Web IDL conversion, mutation records and clone/import state.
The value attribute is independent of the button label. Missing/invalid button
types follow Safari's auto state: command/commandfor presence or a direct select
parent selects button behavior; otherwise the default is submit. Explicit submit,
reset and button types take precedence. Submission eligibility and native input
activation use canonical type state even when script shadows the public property.

Form named-property access (such as a button named reset shadowing form.reset)
remains a separate known gap; these contracts cover attribute reflection.

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/forms/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
