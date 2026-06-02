# Design Conventions

Component naming, token conventions, accessibility rules, and handoff spec format for designers.

---

## Component Audit: Before Designing New

1. Search existing component library for a match
2. Check all variants of the candidate component
3. Only propose a new component if no existing one covers the use case with a variant

**Decision rule:**
- Existing component + new variant → prefer extending
- Entirely new pattern → new component, document in design system

---

## Design Token Naming

```
<category>/<variant>/<scale>

Examples:
color/brand/primary
color/feedback/error
spacing/layout/section
typography/heading/xl
radius/component/card
shadow/elevation/medium
```

Never use raw hex, px values, or font sizes directly — always reference a token.

---

## Required States Per Interactive Element

| State | When |
|---|---|
| Default | Normal, unfocused |
| Hover | Cursor over element |
| Focus | Keyboard or programmatic focus |
| Active / Pressed | Click or tap in progress |
| Disabled | Not interactive |
| Loading | Async operation in progress |
| Error | Validation failure or API error |
| Empty | No data to display |
| Success | Async operation completed |

Document which states apply per component — not all states apply to all elements.

---

## Accessibility Rules (WCAG 2.1 AA minimum)

| Rule | Requirement |
|---|---|
| Text contrast | ≥ 4.5:1 normal text, ≥ 3:1 large text |
| UI component contrast | ≥ 3:1 against adjacent colors |
| Touch targets | ≥ 44×44px on mobile |
| Focus indicators | Visible, high-contrast ring |
| ARIA labels | Required on icon-only buttons, unlabelled fields |
| Keyboard navigation | All interactive elements reachable by keyboard |

Flag any component that cannot meet AA without a design change before handoff.

---

## Handoff Spec Format

```
Feature: <feature name>

Components:
- <ComponentName> — variant: <variant> — source: <existing | new>

States: <list applicable states>

Tokens:
- color: <token-name>
- spacing: <token-name>
- typography: <token-name>

Accessibility:
- ARIA: <label or role>
- Keyboard: <tab order and focus behavior>
- Contrast: <ratio — pass | fail>

Engineering notes:
- <constraints, animations, edge cases>

Out of scope:
- ...
```

---

## Common Pitfalls

- Raw values in specs — always token names
- Missing states — an element without an error state produces inconsistent UX
- New component without audit — always search the library first
- Accessibility as afterthought — check contrast and keyboard nav during design, not at handoff
