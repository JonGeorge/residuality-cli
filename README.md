# Residuality CLI

A small Rust CLI for applying **Residuality Theory** — a software architecture method created by
[Barry M. O'Reilly](https://leanpub.com/u/barrymoreilly) — to a codebase. It keeps your stressor
analysis in plain CSV files committed next to the code, and derives reports (an incidence matrix,
contagion triggers, and an empirical "residual index") from them.

> **Attribution:** Residuality Theory and all of its concepts (stressors, residues, attractors,
> contagion, the incidence matrix, empirical testing) are the work of Barry M. O'Reilly. This tool
> is an independent companion for practicing the method and is not affiliated with or endorsed by
> him. See [References](#references) for the original papers and book.

## What is Residuality Theory?

Traditional architecture methods start from requirements and try to predict what a system must do.
Residuality Theory starts from the opposite direction: it assumes the environment around your
software is a *complex system* — markets, users, regulators, vendors, your own team — that **will**
change in ways you cannot predict, and asks what will be left of your architecture when it does.

The core vocabulary, as defined in O'Reilly's
[2020 paper](https://www.sciencedirect.com/science/article/pii/S1877050920305585):

- **Stressor** — anything the environment can throw at the system: a vendor dies, traffic ×100,
  a law changes, a CSV row is malformed. You don't need it to be *likely*; you need it to be
  *conceivable*.
- **Residue** — what remains of the system after a stressor hits it. "Residuality" is the property
  of having useful residues: when stressed, some part of the system survives and keeps functioning.
- **Attractor** — the state a system naturally falls into under a given stress. Instead of
  preventing stress, you design so that the states the system falls into are acceptable ones.

The method, roughly: propose a naive architecture, bombard it with stressors one at a time, and for
each stressor describe how the business reacts and what technical change would let the system
survive. Each surviving configuration is a residue. Integrating many residues produces an
architecture that holds up under stressors you *never analysed* — because stressor analysis
uncovers the hidden coupling in the system, not because you predicted the future.

Two ideas matter for how this tool works, both developed in the
[2022 follow-up paper](https://www.sciencedirect.com/science/article/pii/S1877050922004975)
([ACM mirror](https://dl.acm.org/doi/abs/10.1016/j.procs.2022.03.084)):

- **The incidence matrix** — a grid of stressors × components, with a 1 where a stressor impacts a
  component. Reading the rows and columns exposes *contagion*: components that fail together, and
  stressors that sweep across the whole system.
- **Empirical testing** — after design, you test the architecture with a *fresh* set of stressors
  it has never seen and measure how many it survives compared to the naive design. This gives an
  evidence-based reason to believe the architecture is more resilient, not just a feeling.

For a proper introduction read O'Reilly's short book,
[*Residues: Time, Change, and Uncertainty in Software Architecture*](https://leanpub.com/residuality)
(Leanpub, 2024) — this project follows its spreadsheet-driven workflow, which is why everything
here is a CSV file.

## What the CLI does

Your architecture lives in two human-editable CSV files (one double-click away from Excel/Sheets):

```
architecture/
  components.csv   # id,name — the parts of your system
  stressors.csv    # id,name,detection,attractor,business_reaction,technical_change,affected_components
reports/
  matrix_<date>.csv  # derived incidence matrix — generated, never hand-edited
```

Each stressor records how you'd *detect* it, the *attractor* it pulls the system toward, the
*business reaction*, the *technical change* that would let the system survive, and which components
it *affects* (semicolon-separated ids). The incidence matrix is always derived from these files,
never stored as editable data.

Deliberately absent, per the method: stressors have **no probability or cost fields** (you stress
the architecture first and worry about likelihood later), and there is no stressor template
library — stressors are specific to your system.

## Usage

Requires a [Rust toolchain](https://rustup.rs) (edition 2024). Build with `cargo build`; the
binary is available as both `residuality` and the shorter `res`.

```sh
# add components
res component add storage "Storage Layer (CSV read/write)"
res component list        # alias: ls

# log a stressor and the components it touches
res stressor add \
  --id malformed_csv \
  --name "Malformed CSV row" \
  --detection "csv::Error on deserialize" \
  --attractor "strict schema validation" \
  --business-reaction "user loses trust in tool output" \
  --technical-change "validate headers and surface row number" \
  --affects "storage;deserializer"
res stressor list

# derive the incidence matrix
res matrix print           # to stdout
res matrix export          # writes reports/matrix_<date>.csv
```

### Command status

| Command | Status |
|---|---|
| `component add` / `list` | working |
| `stressor add` / `list` | working |
| `matrix print` / `export` | working |
| `init` | stub |
| `triggers` (seven contagion triggers) | planned |
| `test <file>` (empirical residual index) | planned |

## References

All concepts implemented here originate with Barry M. O'Reilly:

- O'Reilly, B. M. (2020). [An Introduction to Residuality Theory: Software Design Heuristics for
  Complex Systems](https://www.sciencedirect.com/science/article/pii/S1877050920305585).
  *Procedia Computer Science*, 170, 875–880. (Open access.)
- O'Reilly, B. M. (2022). [Residuality Theory, random simulation, and attractor
  networks](https://www.sciencedirect.com/science/article/pii/S1877050922004975).
  *Procedia Computer Science*, 201.
- O'Reilly, B. M. (2024). [*Residues: Time, Change, and Uncertainty in Software
  Architecture*](https://leanpub.com/residuality). Leanpub. — the practical guide this tool follows.
- O'Reilly, B. M. [*The Architect's Paradox: Uncertainty and the Philosophy of Software
  Architecture*](https://leanpub.com/u/barrymoreilly). Leanpub. — the deeper philosophical companion.
- O'Reilly, B. M. (2021). [“There Is No Spoon” — The Path to Residuality
  Theory](https://www.cutter.com/sites/default/files/APM/2021/The%20Path%20to%20Residuality%20Theory%20Collection.pdf).
  *Cutter Business Technology Journal*.
- Talk: [An Introduction to Residuality Theory — Barry O'Reilly, NDC Oslo
  2023](https://www.classcentral.com/course/youtube-an-introduction-to-residuality-theory-barry-o-reilly-ndc-oslo-2023-213418) (video).

## License

Public domain — see [LICENSE](LICENSE) (The Unlicense). The license covers this tool's code only;
Residuality Theory and the referenced publications remain the intellectual work of their author.
