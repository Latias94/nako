# Error Handling

The current wrapper delegates all CLI parsing and failure handling to UniFFI.

## Required Patterns

- Call `uniffi::uniffi_bindgen_main()` directly unless there is a concrete need
  for wrapper behavior.
- Let UniFFI own argument validation, stderr output, exit behavior, and generated
  artifact errors.
- If wrapper behavior is added, keep errors command-line friendly and avoid
  importing application error types.
- Keep compile validation as the primary local check.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| CLI args are valid | UniFFI generates bindings |
| CLI args are invalid | UniFFI reports the CLI error |
| Binding source fails | UniFFI reports generation failure |
| Wrapper compile breaks | `cargo check -p nako-uniffi-bindgen` fails |

## Forbidden Patterns

- Do not catch and reinterpret UniFFI CLI failures without a tested reason.
- Do not add `anyhow`, application `NakoError`, or server runtime error types.
- Do not hide UniFFI stderr output behind custom logging.
- Do not add panics around CLI argument handling.

## Evidence

- `crates/nako-uniffi-bindgen/src/main.rs`
