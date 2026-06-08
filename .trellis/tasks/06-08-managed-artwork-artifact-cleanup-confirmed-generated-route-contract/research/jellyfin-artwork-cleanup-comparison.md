# Jellyfin artwork cleanup comparison

## Reference Observed

- Local reference: `repo-ref/jellyfin/Emby.Server.Implementations/Chapters/ChapterManager.cs`.
- Relevant behavior: `DeleteDeadImages` computes image paths that are no longer referenced by chapter metadata, filters candidates to supported image extensions, and then deletes each path best-effort with logging on failure.

## Takeaways For Nako

- Jellyfin's cleanup is not a generic "delete whatever path the caller names" operation. The server derives dead image candidates from current metadata state and a supported image-type guard.
- Nako should keep the same architectural posture: Admin clients should request a maintenance command, not submit raw artifact paths, storage URIs, or file names.
- Because Nako's artifact cleanup deletes DB rows and best-effort files, it should be an explicit confirmed mutation before it becomes a generated Admin Web route.

## Mapping To This Task

- Nako already derives cleanup candidates through repository lifecycle state and returns redaction-safe cleanup summaries.
- The missing boundary is HTTP-level confirmation. Adding `confirm=true` aligns cleanup exposure with the already confirmed stray-file remediation route.
- This task should not copy Jellyfin code or data structures. It uses Jellyfin only to validate the design principle: deletion targets stay server-derived and constrained.
