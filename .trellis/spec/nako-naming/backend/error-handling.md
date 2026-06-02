# Error Handling

The parser is total: `parse_path` returns `ParsedName` for every input rather
than a `Result`.

## Required Patterns

- Return an unknown, low-confidence hint for weak evidence instead of failing.
- Treat unparseable optional facts as absent.
- Keep confidence values explicit and test-covered.
- Keep title fallback deterministic; current episode fallback uses `unknown`
  when the title part is empty.
- Preserve evidence source and evidence value so callers can explain why a hint
  was produced.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| `S01E02` token found | Episode hint, confidence `900` |
| `2x03` token found | Episode hint, confidence `880` |
| Year `1888..=2100` found | Movie hint, confidence `760` |
| No strong pattern | Unknown hint, confidence `350` |
| Leading slash path | Trim slash and parse file name |
| Invalid year or number | Ignore field and continue |

## Forbidden Patterns

- Do not panic or unwrap on path shape, extension shape, token parse, or year
  parse.
- Do not return an error for unknown titles.
- Do not silently change confidence values without updating tests and parser
  version expectations.
- Do not treat parser hints as confirmed metadata matches.

## Wrong vs Correct

### Wrong

```rust
let parsed_year = token.parse::<u16>().unwrap();
```

### Correct

```rust
let parsed_year = token.parse::<u16>().ok()?;
```

## Evidence

- `crates/nako-naming/src/lib.rs`
