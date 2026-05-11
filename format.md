# txviz JSON Input Format (Informal Draft)

This document is an **informal working specification** for the JSON files accepted by `txviz`.
It is intended as a practical basis for a later formal schema.

It prioritizes:
1. Core features and common authoring patterns.
2. Fields required for useful diagrams.
3. Ambiguities and edge cases that need future schema decisions.

---

## 1) Scope and Goals

A txviz input file describes a graph of Bitcoin transactions and spending relationships, for rendering into SVG/HTML diagrams.

At a high level, the format covers:
- Transactions
- Inputs and outputs
- Spend relationships between outputs and inputs
- Candidate/hypothetical transactions
- Optional annotations, conditions, style hints, bookmarks, and layout hints

The format is human-authored JSON and is normalized internally before layout/rendering.

---

## 2) Top-level Object

A txviz input file is a JSON object.

### Core top-level key
- `transactions` (array, expected): list of transaction objects.

### Additional top-level keys (mentioned in README, details TBD)
- `bookmarks`
- `layout`
- `styles` or style-class declarations
- scenario/interactive metadata

> Ambiguity: README names several concepts (bookmarks, classes, layout hints) but does not define concrete key names or structures yet.

---

## 3) Transactions (Most Important)

Each element of `transactions` is an object representing either:
1. A **real Bitcoin transaction** (from serialized hex), or
2. A **symbolic template transaction** used for explanation.

### 3.1 Transaction identity (recommended direction)
- `id` (string, recommended): **symbolic, human-readable identifier** unique within the file.

For raw-hex transactions, there are conceptually two identifiers:
1. **Symbolic ID**: `id` (author-chosen label like `funding_tx`).
2. **Bitcoin TXID**: computed from the serialized transaction according to Bitcoin consensus/standard rules.

Recommended rule: `id` is always symbolic, never overloaded to mean the computed TXID.
Additional rule: symbolic `id` values **must not** be exactly 64 hex characters (`[0-9a-fA-F]{64}`), to prevent namespace collisions with TXID-style references.

### 3.2 Transaction status
- `status` (string, optional)
  - Known value from README example: `"candidate"`.

Expected meaning:
- `candidate`: hypothetical or not-yet-confirmed path shown in diagram.

> Ambiguity: full enum is undefined (e.g., confirmed/unconfirmed/invalid/replaced/etc.).

### 3.3 Transaction representation mode
A transaction may be authored in one of two modes:

#### A) Symbolic mode
Provide explicit `inputs`/`outputs` and optional descriptive metadata.

#### B) Raw mode
Provide serialized transaction hex; tool parses/deserializes and infers structure.

> Ambiguity: exact key for raw hex is not defined in README (likely something like `raw_hex`, `hex`, or `tx_hex`).


### 3.4 Transaction header fields (optional)
- `version` (integer, optional): exact encoded transaction version.
- `locktime` (optional):
  - integer: exact encoded consensus locktime value, or
  - object form `{ "height": <int> }`, or
  - object form `{ "time": <int> }`.

Recommended normalization for `locktime`:
- Preserve original author form for display/tooling when useful.
- Resolve to a single numeric consensus value internally for rendering logic.

---

## 4) Outputs

`outputs` is an array of output objects attached to a transaction.

### Common output fields
- `amount_sat` (integer, optional): concrete output value in satoshis.
- `amount_expr` (string, optional): symbolic/mathematical amount description (e.g. `"2 * N"`).

Authoring recommendation:
- Authors SHOULD provide either `amount_sat` or `amount_expr` for each output.
- If both are present for an output, `amount_sat` should be preferred for primary display.
- When both are present, `amount_expr` should still be shown as secondary context (e.g., tooltip/annotation).

Output index is positional in the `outputs` array (`0`, `1`, ...), and is used by references like `txid:0`.

> Ambiguity:
- Exact rendering contract for showing `amount_expr` alongside `amount_sat`.
- Whether alternative key names (e.g. `amount_desc`) should be supported.
- Handling for omitted amount data in real tx parsed from hex (likely inferred).

---

## 5) Inputs and Spend Relationships

`inputs` is an array of input objects attached to a transaction.

### Common input fields
- `spends` (string, common): reference to a previous transaction output.
- `sequence` (optional):
  - integer: exact encoded input sequence, or
  - object form similar to locktime shorthands (e.g. height/time-oriented expression), normalized to a concrete integer sequence value.

Reference syntax shown in README:
- `"<ref>:<output-index>"`, e.g. `"parent:0"`.

This implies each input consumes exactly one prior output, and forms an edge in the graph.

### 5.1 Proposed reference namespace rule
To support both machine-stable and human-friendly references, a referent may be:
- Symbolic ID: `<symbolic-id>:<n>`
- TXID reference: `<txid-hex>:<n>`

Where `<txid-hex>` is the real computed transaction id for raw-hex transactions.

### Reference constraints (proposed)
- `<output-index>` is zero-based and in bounds.
- Symbolic refs resolve against transaction `id`.
- TXID refs resolve against computed txids for raw-hex transactions.
- If a token could match both symbolic and txid namespaces, schema must define precedence (recommended: reject as ambiguous).

> Ambiguity still to settle:
- TXID refs are implicit by hex-pattern matching; no explicit tag is required.
- Whether forward references are allowed (spending a tx defined later in array).
- Whether one output can be referenced by multiple inputs (double-spend modeling) and how that is rendered.
- Whether alternate structured reference objects are supported in addition to string shorthand.

---

## 6) Minimal Valid Authoring Pattern

Canonical minimal symbolic pattern:

```json
{
  "transactions": [
    {
      "id": "parent",
      "outputs": [{ "amount_sat": 100000 }]
    },
    {
      "id": "refund",
      "status": "candidate",
      "inputs": [{ "spends": "parent:0" }]
    }
  ]
}
```

This is sufficient to express a simple parent→child spend edge.

---

## 7) Normalization Expectations

Before layout/rendering, input is normalized:
- Defaults applied
- References resolved
- Spend edges generated
- Raw Bitcoin transactions deserialized/analyzed

Implications for authors:
- Shorthand forms may be accepted and expanded.
- Internal representation is stricter than author input.

> Ambiguity: no public contract yet for which defaults exist and where conflicts are rejected vs silently normalized.

---

## 8) Annotation/Condition Metadata (Important, Partially Specified)

README indicates support for:
- Annotations on transactions/inputs/outputs/edges
- Conditions and satisfaction data (e.g., timeout or script-path explanation)
- Tooltips
- Semantic highlighting and hidden details

These likely map to optional metadata fields on graph elements.

Minimal annotation fields (recommended):
- `title` (string, optional) on transactions, inputs, and outputs.
- `description` (string, optional) on transactions, inputs, and outputs.
- `descriptor` (string, optional) on outputs and on spend connections/edges.

Descriptor handling rule (recommended):
- `descriptor` values are treated as display strings only.
- They are **not** parsed or validated for semantic correctness by txviz.
- Abstract or schematic values such as `pk(K)` are therefore allowed.

> Ambiguity:
- Beyond `title`/`description`/`descriptor`, additional annotation keys and nesting are undefined.
- Whether conditions are plain text, structured objects, or both.
- Whether hidden details affect rendering only or also normalization.

---

## 9) Styling Classes

README mentions styling classes and semantic highlighting.

Expected capabilities:
- Attach classes/tags to transactions, inputs, outputs, edges.
- Possibly define class-to-style mappings at top level.

> Ambiguity:
- No defined grammar for classes (single string vs string array).
- No documented precedence rules between inline styles and class styles.

---

## 10) Layout Hints and Bookmarks

README mentions layout hints and bookmarks, especially for interactive navigation.

Expected usage:
- Layout hints: nudging graph arrangement/grouping.
- Bookmarks: named navigation targets in interactive mode.

> Ambiguity:
- No stable key names/shape for these objects yet.
- Unknown behavior in static SVG-only output when bookmarks/hints are present.

---

## 11) Suggested Validation Rules for Future Schema

To reduce author confusion, a formal schema should likely enforce:
- Top-level object type and known keys.
- `transactions` present and array-typed.
- Duplicate transaction `id` values are ignored after first occurrence (first definition wins).
- `spends` references parse as `<ref>:<index>` where `<ref>` is symbolic id or txid.
- Referenced transactions/outputs exist.
- Output index is non-negative integer in bounds.
- Optional strict mode for ambiguous fields.

---

## 12) Edge Cases and Ambiguities to Resolve (Lower Priority)

1. **Duplicate symbolic IDs**: ignore duplicates after first definition (first-wins).
2. **Missing outputs on referenced tx**: implicit unknown output vs validation error.
3. **Dangling spends references**: hard error vs warning.
4. **Cycle handling** in symbolic graphs.
5. **Coinbase-like inputs** with no `spends` reference.
6. **Multiple competing spends** of same output (intentional branch vs invalid double-spend).
7. **Amount handling** for symbolic outputs without values.
8. **Large integers** and JSON number precision concerns for sats.
9. **Raw+symbolic mixed transaction**: whether allowed in same tx object.
10. **Hex-like symbolic IDs**: should be a hard validation error if exactly 64 hex chars.
11. **Unknown keys policy**: ignored, warned, or rejected.

---

## 13) Authoring Guidance (Interim)

Until formal schema exists:
- Always provide explicit symbolic `id` for each transaction, including raw-hex transactions.
- Prefer symbolic refs in `spends` (`<id>:<index>`) for readability; allow txid refs when needed for precision.
- Keep output arrays explicit when they will be referenced.
- Prefer integers for satoshi amounts.
- Treat candidate/hypothetical paths with `status: "candidate"`.
- Keep optional metadata additive (do not rely on undocumented behavior).

---

## 14) Versioning Proposal (For Future)

Consider adding a top-level format version, e.g.:
- `"format_version": "0.x"`

This would allow controlled evolution toward a concrete schema without breaking existing examples.


---

## 15) Symbols, Keys, and Legend (Future)

The format should later define a top-level key/legend section for symbolic definitions used in annotations and expressions, such as:
- Amount symbols (e.g. `N`)
- Key placeholders (e.g. `K`)
- Other protocol-specific shorthand

This would let expressions like `2 * N` and descriptors like `pk(K)` link to explicit human-readable definitions.
