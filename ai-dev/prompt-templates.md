# Prompt Templates — Tissot

Reusable prompts for AI-assisted development on this project.

## New Rule Implementation

```
Read CLAUDE.md, ai-dev/architecture.md, ai-dev/guardrails/coding-standards.md.
Read ai-dev/agents/rust_expert.md for Rust patterns.

Implement a new checker rule: [RULE_ID] in the [DOMAIN] domain.

The rule should check: [DESCRIPTION OF WHAT IT CHECKS]

Before writing code:
1. Confirm you understand the rule's purpose
2. List the files you will create or modify
3. Describe the algorithm
4. Identify test cases (at least: known-good, known-bad, edge case)
5. Show me the plan

Do not proceed until I type Engage.
```

## X-Ray Feature Extension

```
Read CLAUDE.md, ai-dev/architecture.md, specs/projection-xray.md.
Read ai-dev/guardrails/coding-standards.md.

Extend the X-Ray engine: [DESCRIPTION]

Remember:
- X-Ray is the hero feature — quality and visual impact matter above all
- All outputs must render correctly in the MapLibre visual report
- Performance target: < 2 seconds for 10K features
- Show the plan before coding
```

## Visual Report Development

```
Read CLAUDE.md, ai-dev/architecture.md.
Read ai-dev/guardrails/coding-standards.md (HTML/JS section).

Build/modify the visual report for: [COMMAND — xray/check/diff/score/watch]

Requirements:
- Self-contained HTML (no CDN dependencies)
- MapLibre GL JS for map rendering
- Dark theme default
- Works offline
- Responsive layout (laptop screen minimum)
- Data passed as embedded JSON in the HTML template

Show the template structure before writing code.
```

## Code Review

```
Read CLAUDE.md, ai-dev/patterns.md, ai-dev/guardrails/coding-standards.md.

Review [FILE OR MODULE] for:
- Adherence to coding standards in guardrails
- Error handling completeness (no unwrap in library code)
- Proper use of geo/proj crate types
- Visual-first principle (does this finding carry geometry for map rendering?)
- Performance implications for large datasets
- Test coverage

Produce a numbered list of findings with severity (Critical / Warning / Info).
```

## End-of-Session Commit

```
Read CLAUDE.md.

Summarize all changes made this session.
Group into logical git commits.
Use format: feat(module): description or fix(module): description

Suggested commits should follow this module naming:
- xray: X-Ray engine changes
- check: Checker engine changes
- fix: Fix engine changes
- score: Score engine changes
- io: IO layer changes
- report: Visual report changes
- cli: CLI changes
- python: Python binding changes
- docs: Documentation changes

Show proposed commits. Do not run git until I type Engage.
```
