---
name: code-style-python
rule_number: "01"
scope: all python source
enforcement: ruff format + lint, pyright strict, pre-commit
---

# Rule 01: Python Code Style

Every `.py` byte in lewdzone-launcher must pass ruff (format + lint) and pyright
strict. Style is enforced by tooling, not by taste.

## Tooling baseline

| Tool | Config anchor | Enforced |
| --- | --- | --- |
| `ruff format` | default black-compatible | formatting |
| `ruff check` | `select = ["E","F","I","UP","B","SIM","C4","S"]`-style curated set | lint |
| `pyright --strict` | `pyproject.toml [tool.pyright]` | types |
| `pre-commit` | runs ruff + pyright on staged files | hook |

## Format rules

1. Black-compatible: 88-char line length, double quotes, trailing commas in
   multi-line collections and calls.
2. Import section order: stdlib → third-party → local (`lewdzone_launcher`), one
   blank line between groups; `isort` via ruff.
3. No unused imports/variables (`F401`, `F841`).
4. Wrap lines with parentheses, not backslashes.

## Naming (see also Rule 02)

- Modules/files: `snake_case`.
- Packages: lowercase, no underscores (`lewdzone_launcher`, packages inside are
  `scraper`, `resolver`, `db`, `fdm`, `cli`, `gui`).
- Classes: `PascalCase`. Functions/methods/variables/params: `snake_case`.
- Constants: `UPPER_SNAKE`.
- Private module members: leading `_`; never double underscore in names except
  dunders.
- Type aliases: `PascalCase` with `TypeAlias` annotation.

## Types (pyright strict)

1. Every public function/method has full annotations for params and return.
2. Module-level and class-level annotated attributes; typed dataclasses for
   domain models (see Rule 02/03 canonical models).
3. No bare `Any` in public signatures. No `# type: ignore` without an
   accompanying `# reason:` comment; keep count near zero.
4. Use `NewType` or `TypeAlias` for domain primitives where unit confusion is
   possible (e.g. `GameId = NewType("GameId", int)`, `PostId`).
5. `Optional` vs `None` default: prefer explicit `= None` default with
   `| None`.
6. Discriminated unions (`Literal`/`TypeVar`/dataclass union) preferred over
   flag booleans for modes (e.g. `SortMode`, `Platform`, `TabId`).

## Anti-patterns (fail lint)

```python
# BAD: global mutable, bare except, magic parse, stringly-typed
state = {}
try:
    ...
except:
    ...
x = url.split("#t=")[1].split(".")          # stringly-typed token handling
def handle(thing): ...                        # untyped
```

```python
# GOOD
@dataclass(frozen=True)
class DownloadEntry:
    platform: Platform
    tab: DownloadTab
    label: str
    token: GoToken                     # typed domain value object
```

## Verify

- `ruff format --check .`
- `ruff check .`
- `pyright --verifytypes lewdzone_launcher`
- `pre-commit run --all-files`