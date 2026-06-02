# Figma Skill

Conventions for reading, auditing, and producing design artifacts in a Figma-based design workflow.

---

## Stack Overview

- **Design tool:** Figma (components, design tokens, prototypes)
- **Integration:** Figma MCP server (`mcp__claude_ai_Figma__*`)
- **Output:** Handoff specs consumed by frontend-developer agent

---

## Design System Concepts

| Concept | Definition |
|---|---|
| **Component** | Reusable UI element with defined variants and states |
| **Token** | Named design value (color, spacing, typography, radius, shadow) |
| **Variant** | Alternate appearance of a component (size, state, theme) |
| **Frame** | A screen or layout container — maps to a route or modal |
| **Auto Layout** | Figma's flex-like layout system — always use over manual positioning |

---

## Component Audit: Before Designing New

1. Search existing component library for a match
2. Check all variants of the candidate component
3. Only propose a new component if no existing one covers the use case with a variant

**Decision rule:**
```
existing component + new variant → prefer extending
entirely new pattern            → new component, document in design system
```

---

## States: Required for Every Interactive Element

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

## Design Tokens: Naming Convention

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

- Never use raw hex, px values, or font sizes directly in specs — always reference a token
- Token names must match the frontend implementation (coordinate with frontend-developer)

---

## Accessibility Rules (WCAG 2.1 AA — Minimum)

| Rule | Requirement |
|---|---|
| Text contrast | ≥ 4.5:1 for normal text, ≥ 3:1 for large text (18px+ or 14px+ bold) |
| UI component contrast | ≥ 3:1 against adjacent colors |
| Touch targets | ≥ 44×44px on mobile |
| Focus indicators | Visible, high-contrast ring on all interactive elements |
| ARIA labels | Required on icon-only buttons, form fields without visible labels |
| Keyboard navigation | All interactive elements reachable and operable by keyboard |

Flag any component that cannot meet AA contrast without a design change.

---

## Handoff Spec Format

```
Feature: <feature name>
Frame: <Figma frame name or link>

Components:
- <ComponentName> — variant: <variant> — source: <existing | new>

States implemented:
- Default, Hover, Error, Empty  (list only applicable states)

Tokens:
- color: <token-name>
- spacing: <token-name>
- typography: <token-name>

Accessibility:
- ARIA: <label or role for each non-obvious element>
- Keyboard: <tab order and focus behavior>
- Contrast: <ratio — pass | fail — action if fail>

Engineering notes:
- <Any constraint, animation spec, or edge case the developer must know>

Out of scope:
- <Explicitly excluded>
```

---

## MCP Tool Usage

Authenticate before accessing Figma files:
1. `mcp__claude_ai_Figma__authenticate` — initiate auth
2. `mcp__claude_ai_Figma__complete_authentication` — confirm

Use file keys from Figma URLs: `figma.com/file/<key>/...`

---

## Common Pitfalls

- **Raw values in specs:** Never specify `#3B82F6` or `16px` — always a token name
- **Missing states:** An element without an error state will have an inconsistent error UX
- **New component without audit:** Always search the library first
- **Accessibility afterthought:** Check contrast and keyboard nav during design, not at handoff
- **Ambiguous spacing:** Use Auto Layout with token-referenced gaps — never manual positioning

---

## See Also

- `docs/conventions/design.md` — full rule set
- `docs/conventions/frontend.md` — token implementation in Angular
- `agentic-setup/agents/designer.template.md` — designer agent behavior
