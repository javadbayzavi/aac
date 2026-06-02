# Product Conventions

User story format, acceptance criteria structure, and decomposition rules for product managers.

---

## User Story Format

```
Title: <verb> <outcome> as <persona>

As a <persona>,
I want to <action>,
So that <outcome>.

Acceptance Criteria:
- Given <context>, When <action>, Then <result>
- ...

Out of scope:
- ...
```

---

## Decomposition Rules

- One story = one deliverable outcome — no compound stories
- Each story must be implementable in ~45 minutes of engineering effort
- Acceptance criteria must be testable, not aspirational
- Any story requiring a design decision before engineering starts → flag with `design` label
- Adjacent ideas → park as follow-up stories, never append to current scope

---

## Acceptance Criteria Rules

- Written in Given/When/Then format only
- Must describe observable behaviour — not implementation detail
- Minimum one criterion per story; no upper limit
- Edge cases and error states count as separate criteria

---

## Epic Structure

```
Epic: <product capability>
└── Story: <user-facing outcome>
    └── Acceptance criteria: <testable behaviours>
```

Every story must belong to an epic. No orphan stories.

---

## Common Pitfalls

- **Compound stories:** "User can register and log in" → two stories
- **Vague criteria:** "The system should be fast" → not testable, rewrite with a measurable threshold
- **Missing out of scope:** without it, scope creep is invisible
- **No epic link:** orphan stories disappear in planning
