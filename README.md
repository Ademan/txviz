# txviz

**txviz** is a tool for creating high-quality diagrams of Bitcoin transaction graphs.

It can visualize both:

1. **Real Bitcoin transactions** parsed from serialized transaction hex.
2. **Symbolic transaction templates** used to explain protocols, presigned transaction sets, and alternative spending paths.

The output is a self-contained SVG for static use in documentation, with optional JavaScript-enhanced interactivity for tooltips, pan/zoom, and guided navigation.

---

## Motivation

Bitcoin protocols are often described as sets of interdependent transactions:

* Bitcoin Whitepaper payment flows
* Mastering Bitcoin examples
* Lightning Network commitment and HTLC transactions
* Ark round and out-of-round transactions
* Assurance Contracts presigned refund paths
* Vaults, escrows, and covenant constructions

These systems are difficult to understand from prose alone. Existing diagram tools are either too generic or require large amounts of manual work.

txviz provides a Bitcoin-specific format for describing transactions and spending relationships, and renders them as publication-quality diagrams.

---

## Features

* Parse real Bitcoin transactions from raw hex.
* Define symbolic transactions with human-readable identifiers.
* Visualize both actual and hypothetical spending paths.
* Annotate transactions, inputs, outputs, and edges.
* Display conditional spending requirements in tooltips.
* Highlight important inputs and outputs with semantic styling.
* Hide irrelevant details while preserving them in tooltips.
* Generate self-contained SVG output.
* Optional JavaScript enhancements:

  * Pan and zoom
  * Bookmarks
  * Guided navigation
  * Rich tooltips
  * Scenario selection

---

## Example Use Cases

* Explaining Pay-to-Contract constructions.
* Visualizing presigned transaction trees.
* Documenting CheckTemplateVerify covenant designs.
* Teaching Lightning Network transaction flows.
* Illustrating Ark protocols.
* Rendering assurance contract transaction sets.
* Producing static images for papers, blog posts, and presentations.

---

## Example Diagram

```text
parent
 └─ output 0
     ├─ child      (cooperative spend)
     └─ refund     (after 144 blocks)
```

In the interactive view, hovering over the refund path may show:

> Spendable by Alice alone after 144 blocks via the timeout script path.

---

## Input Format

txviz accepts JSON describing:

* Transactions
* Inputs and outputs
* Spend relationships
* Candidate transactions
* Conditions and satisfaction data
* Styling classes
* Bookmarks
* Layout hints

Transactions may be specified either as:

* Raw serialized transaction hex, or
* Symbolic templates.

### Minimal Example

```json
{
  "transactions": [
    {
      "id": "parent",
      "outputs": [
        { "amount_sat": 100000 }
      ]
    },
    {
      "id": "refund",
      "status": "candidate",
      "inputs": [
        { "spends": "parent:0" }
      ]
    }
  ]
}
```

---

## Output Formats

### SVG

The primary output format is a self-contained SVG.

The SVG is fully useful without JavaScript and suitable for:

* README files
* Blog posts
* Technical documentation
* Academic papers
* Conversion to PNG or PDF

### HTML

An optional HTML wrapper provides enhanced interactivity using the same underlying SVG.

### PNG/PDF

Static raster and document exports may be generated using external tools.

---

## Command Line Interface

```bash
txviz render example.json -o example.svg
txviz render example.json --html -o example.html
txviz render example.json --dump-normalized
```

---

## Architecture

txviz is structured as a pipeline:

```text
Author Input
    ↓
Normalization
    ↓
Layout
    ↓
Rendering
```

### Normalization

The input is expanded into a fully explicit internal representation:

* Defaults are applied.
* References are resolved.
* Spend edges are generated.
* Bitcoin transactions are deserialized and analyzed.

### Layout

Transactions are arranged automatically, potentially using [Graphviz](https://graphviz.org?utm_source=chatgpt.com) or another layout engine.

### Rendering

The normalized graph is rendered to SVG, optionally with embedded JavaScript.

---

## Design Principles

* **Bitcoin-specific** rather than general-purpose.
* **Static-first** output that remains useful without JavaScript.
* **Progressive enhancement** for richer interaction.
* **Human-authored** input files.
* **Publication-quality** diagrams.
* **Deterministic** rendering.

---

## Project Status

txviz is an experimental project under active development.

The initial focus is:

1. JSON schema definition
2. Transaction normalization
3. Automatic layout
4. Static SVG rendering
5. Interactive enhancements

---

## Planned Features

* Support for PSBT visualization
* Witness and script introspection
* Scenario switching
* Automatic legends
* Theme support
* Additional layout engines

---

## License

MIT

---

## Contributing

Contributions, suggestions, and example diagrams are welcome.

---

## Why “txviz”?

`txviz` stands for **transaction visualization**.

The project is intended to make complex Bitcoin transaction relationships understandable through clear, interactive diagrams.
