# DL-001: Project Name and Positioning

**Date:** 2026-03-07
**Status:** Accepted
**Author:** Chris Lyons

## Context

Needed a name for a unified geospatial diagnostics tool combining cartographic linting, projection analysis, spatial diffing, and data quality checks. The name must be available on crates.io, PyPI, and GitHub with no conflicts in the geospatial software space.

## Decision

Name the project **Tissot**, after Nicolas Auguste Tissot and his indicatrix (1859) — the ellipses used to visualize map projection distortion. This directly connects to the hero feature (Projection X-Ray) and signals deep cartographic knowledge.

## Alternatives Considered

- **GeoLint** — Rejected: name taken by British Library Java project (bl-dpt/geolint), a Polish surveying company (geo-lint.pl), and an SEO linting tool (geo-lint on npm). Also collides conceptually with the GeoLinter academic paper (2023).
- **cartoscan** — Clean across registries, but less distinctive and doesn't connect to a specific concept.
- **geodx** — Clean on crates.io/PyPI but conflicts with geodx.de, a German surveying software company. Same industry = too risky.
- **splint** — Taken on crates.io (compressed bitmap crate).

## Consequences

- The name "Tissot" shares a name with Tissot SA, a Swiss watch manufacturer. This is a completely different industry with no geospatial overlap. In GIS/cartography contexts, "Tissot" unambiguously refers to the indicatrix.
- The hero feature (Projection X-Ray with Tissot ellipses) provides a direct, intuitive connection between the tool name and its primary function.
- The name works well as a CLI command: `tissot xray`, `tissot check`, `tissot diff`.

## References

- Tissot's indicatrix: https://en.wikipedia.org/wiki/Tissot%27s_indicatrix
- Name availability verified: crates.io, PyPI, GitHub (2026-03-07)
