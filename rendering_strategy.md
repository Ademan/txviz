# Transaction Rendering Strategy (Inputs/Outputs + Context Metadata)

This strategy defines a first-pass layout and rendering contract for transactions where inputs and outputs each contain:

1. **Always-displayed data** (shown directly in the box), and
2. **Contextual metadata** (retained for later tooltip/interactive rendering).

It focuses on deterministic static SVG layout.

---

## 1) Layout Model and Coordinate System

Each transaction renders as a **transaction box** with optional title above it.

Within a transaction box:
- Inputs are arranged as a vertical list aligned to the **left side**.
- Outputs are arranged as a vertical list aligned to the **right side**.
- Inputs and outputs are independently and evenly spaced vertically.

All visible primitives for now use:
- `fill: none`
- `stroke: black`
- `stroke-width: 1px`

---

## 2) Configurable Rendering Parameters

Define a `RenderConfig` (names illustrative):

- `tx_min_width: f32`
- `tx_max_width: f32`
- `tx_title_gap: f32` (distance between transaction title baseline and tx box top edge)
- `io_top_margin: f32` (vertical margin from tx top content origin to first IO row)
- `io_bottom_margin: f32` (vertical margin from last IO row to tx bottom content origin)
- `input_row_gap: f32`
- `output_row_gap: f32`
- `output_pad_x: f32`
- `output_pad_y: f32`
- `io_h_margin: f32` (minimum horizontal gap between right edge of input lane and left edge of output lane)
- `output_h_margin: f32` (minimum horizontal gap between output title and output value)
- `tx_title_max_lines: usize` (default `3`)
- `tx_title_wrap_strategy` (word-wrap with ellipsis on overflow)
- `font_*` metrics settings (or measured metrics provider)

Optional first-pass defaults can be introduced in code later; this document defines behavior, not numeric defaults.

---

## 3) Data Partitioning: Visible vs Contextual

For each transaction/input/output, split fields into:

- **Always displayed** (rendered now).
- **Contextual metadata** (stored in layout/render nodes for future tooltip/interactive use).

### Phase-1 visible fields
- Transaction:
  - `annotations.title` (if present), rendered above tx box.
  - `version` and `locktime` (if both or either specified), rendered on one line inside tx box.
- Output:
  - Output title: left-justified in output box.
  - Output value (if present): right-justified in output box.

### Phase-1 intentionally hidden fields
- Transaction description.
- Output description.
- Any additional contextual metadata.

---

## 4) Sizing Rules

## 4.1 Input box width/height (uniform per transaction)

All inputs on a given transaction share one width.

Per input, compute a required row width from its always-visible fields (phase-1 may use placeholder/minimum content rules until input text fields are finalized). Then compute:

- `input_w = max(required_input_width_i for all inputs i)`
- `input_h` may be constant or per-row by content policy; phase-1 should prefer a constant row height for visual consistency.

Input box geometry then uses this shared `input_w` for every input row in the transaction.

## 4.2 Output box width/height (uniform per transaction)

Each output’s required width should be sized to fully contain its visible text:

- Let `title_w` be measured width of output title text.
- Let `value_w` be measured width of output value text (0 if absent).
- Content width:
  - if value present: `title_w + output_h_margin + value_w`
  - else: `title_w`
- Required per-output width:
  - `output_required_w(j) = output_pad_x + content_width + output_pad_x`
- Output box height:
  - `output_h = output_pad_y + text_line_height + output_pad_y`

All outputs on a given transaction share one width:

- `output_w = max(output_required_w(j) for all outputs j)`

Then each output row uses this shared `output_w`.

Text anchors:
- Title x = `output_box_left + output_pad_x`, anchor=start.
- Value x = `output_box_right - output_pad_x`, anchor=end.

## 4.3 Transaction width

Transaction width should track title width while clamped:

1. Measure/wrap transaction title if present.
2. Compute wrapped line widths using max text width budget derived from `tx_max_width`.
3. Limit to `tx_title_max_lines` (default 3).
4. If wrapped text exceeds max lines, abbreviate last visible line with ellipsis.
5. Let `title_w` be the maximum width among the rendered title lines (or 0 if no title).
6. Compute preferred width: `preferred_tx_w = title_w + 2 * tx_horizontal_title_padding`.
7. Clamp:
   - `tx_w = clamp(preferred_tx_w, tx_min_width, tx_max_width)`.

Additionally, tx width must be at least large enough for mandatory internal rows (e.g., version/locktime row or future constraints). In practice:

- `min_required_internal_content_width` should include shared row widths:
  - left input lane using `input_w`
  - right output lane using `output_w`
  - required lane separation `io_h_margin` between input and output lanes
  - any center/header constraints
- `tx_w = max(tx_w, min_required_internal_content_width)`
- then re-clamp to max bound where needed.

Explicitly:

- `min_required_internal_content_width >= input_inset_left + input_w + io_h_margin + output_w + output_inset_right`

If the current width would place input and output lanes closer than `io_h_margin`, increase `tx_w` until the minimum separation is satisfied, up to `tx_max_width`.

(If constraints conflict, use max bound and allow overflow handling policy to be defined later.)

---

## 5) Vertical Flow and Offsets

Define a transaction-local content Y origin at tx top edge.

1. Compute `header_row_h`:
   - `header_row_h = text_line_height` when version and/or locktime are shown.
   - `header_row_h = 0` otherwise.
2. Compute `title_block_h`:
   - `title_block_h = rendered_title_line_count * text_line_height + (rendered_title_line_count - 1) * title_line_gap`
   - Title is outside the tx box and does not increase box height; it only affects overall transaction node bounds.

3. Compute top offset for IO rows:
   - `io_y_start = header_row_h + io_top_margin`

4. Apply **top-margin collapse behavior** when header exists:
   - The requirement says header height should push IO down, and top margin should collapse into this offset.
   - Implement as:
     - `effective_io_top_offset = max(header_row_h, io_top_margin)`
   - Then:
     - `io_y_start = effective_io_top_offset`

This prevents double-spacing when both header row and top margin are non-zero while still ensuring the IO rows sit below header content.

5. Input and output row tracks:
   - Inputs: `y_in(i) = io_y_start + i * (input_h + input_row_gap)`
   - Outputs: `y_out(j) = io_y_start + j * (output_h(j) + output_row_gap)`

6. Transaction content bottom:
   - `content_bottom = max(last_input_bottom, last_output_bottom)`

7. Transaction box height:
   - `tx_h = content_bottom + io_bottom_margin`

---

## 6) Horizontal Placement Rules

- Inputs are positioned relative to the **left edge** of tx box:
  - `input_left = tx_left + input_inset_left`
- Outputs are positioned relative to the **right edge** of tx box:
  - `output_right = tx_right - output_inset_right`
  - `output_left = output_right - output_w`

This preserves stable edge anchoring even as tx width changes.

---

## 7) Even Spacing Semantics

“Evenly spaced and arranged vertically” is satisfied by constant per-side row gaps:

- Inputs use `input_row_gap`.
- Outputs use `output_row_gap`.

If a side is empty, no rows are drawn and only margins/header contribute.

---

## 8) Rendering Order (SVG)

Per transaction draw in this order (prefer grouped `<g>` hierarchy):
1. `<g class="tx">`
2. `  <g class="tx-title">` transaction title text (if any), immediately above tx box.
3. `  <g class="tx-body">`
4. `    <rect class="tx-box">`
5. `    <g class="tx-header">` version/locktime row (if specified).
6. `    <g class="tx-inputs">` input row `<g>` nodes with box + text.
7. `    <g class="tx-outputs">` output row `<g>` nodes with box + title/value text.
8. `  </g>`
9. `</g>`

This layering keeps strokes visible and text legible.

---

## 9) Suggested Implementation Phases

### Phase A: Layout structs
- Add layout structs (`TxLayout`, `InputLayout`, `OutputLayout`) carrying:
  - geometry (`x`, `y`, `w`, `h`)
  - visible text placements
  - opaque metadata handles for contextual info.

### Phase B: Measurement adapter
- Introduce `TextMeasurer` trait to measure text widths/heights deterministically.
- For early SVG-only mode, start with a simple monospace approximation, later replace with font-accurate measurement.

### Phase C: Transaction local layout function
- `layout_transaction(tx, config, measurer) -> TxLayout`
- Implement sizing and offsets exactly as above.

### Phase D: SVG emitter
- Render boxes/text from layout output.
- Keep contextual metadata attached as `data-*` attributes for future tooltip support.

---

## 10) Edge Cases to Handle Deterministically

- Very long transaction title:
  - Wrap lines up to `tx_title_max_lines` (default 3), then ellipsize overflow on final line.
- Missing output value:
  - Render only left-justified title.
- Empty title and empty value output:
  - Render minimal padded box or skip (choose policy; recommend render minimal box for index stability).
- Header-only transaction (no IO rows):
  - Height still includes header + bottom margin.
- Asymmetric counts (many inputs, few outputs):
  - Transaction height follows taller side.

---

## 11) Test Matrix (Recommended)

- No title, no header, 1 input / 1 output.
- Title present with min/max width clamping.
- Version+locktime present and top-margin collapse behavior.
- Outputs with and without value text.
- Many outputs with varying text widths (verify right-edge anchoring).
- Different `input_row_gap` / `output_row_gap` settings.
- Zero inputs or zero outputs.

This strategy gives deterministic geometry now, while preserving the data model needed for later interactive metadata rendering.
