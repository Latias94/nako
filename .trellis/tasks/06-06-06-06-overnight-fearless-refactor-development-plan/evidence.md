# Evidence

## 2026-06-07 M1 / Storage Planning

- Created and committed the continuous development plan:
  `1ce7cd0f chore(task): plan overnight fearless refactor development`.
- Recorded completed M1 evidence gates in roadmap/goal docs:
  `f8eac5fe docs(roadmap): record completed M1 evidence gates`.
- Recorded OpenDAL adapter-spike decision instead of adding a production
  dependency:
  `c035a813 docs(storage): record OpenDAL adapter spike decision`.

## 2026-06-07 VFS Byte Range Refactor

Commits:

- `07743077 refactor(vfs): centralize byte range validation`
- `4d1ec9d1 fix(vfs): validate open-ended WebDAV range length`
- `0f58cc0e fix(vfs): reject invalid WebDAV range syntax`

What changed:

- Moved shared byte-range boundary validation onto `ByteRange`.
- Removed duplicated local/WebDAV range validation helpers.
- Fixed WebDAV open-ended range reads so `bytes=1-` expects `object_len - 1`
  bytes rather than the whole object length.
- Added syntax validation before WebDAV constructs a `Range` header, so
  zero-length and overflowing ranges reject before sending invalid remote
  requests when object metadata lacks length.

Validation:

- `cargo nextest run -p nako-vfs byte_range --no-fail-fast`
  - Result: passed, 9 tests run after syntax-validation slice.
- `cargo nextest run -p nako-vfs webdav_backend_reads_open_ended_byte_ranges_with_resolved_length --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-vfs --no-fail-fast`
  - Result after open-ended fix: 58 tests passed.
  - Result after syntax-validation fix: 59 tests passed.
- `cargo check -p nako-vfs --tests`
  - Result: passed after both VFS range fixes.
