# PR Workflow

Applies to all code-producing roles (BE, FE). Load this alongside your stack convention.

---

## Git Strategy

- Trunk-based. `main` is the only long-lived branch.
- **Avoid `git cherry-pick`.** If a change is needed on two branches, the branching strategy is wrong — merge from trunk or rebase instead.

---

## Branch Naming

`<issue>-<role>-<short-description>` — e.g. `116-be-taxonomy-domain`, `116-be-taxonomy-rest`, `116-fe-taxonomy-ui`.

- `<role>` is `be`, `fe`, or `devops` — prevents parallel dispatch collisions on remote.
- `<short-description>` reflects the chunk's content, not a sequence number.

---

## One feature = one branch = multiple PRs

- Branch name: `<issue-number>-short-kebab-slug` (e.g. `42-add-booking-form`).
- **Hard size limits — both must hold:** ≤ 200 lines changed (added + removed) **and** ≤ 4 new files. Exceeding either = split the PR (Stacked PRs). No exceptions except greenfield module bootstrap (note it explicitly in the PR description).
- Size is scoped against the approved feature plan. If you find yourself outside scope, return `WORK_BLOCKED` to the orchestrator.
- All modified files must be covered with tests in the same PR. No drive-by test additions in separate PRs.

---

## Commit Messages

- Imperative mood, ≤ 70 chars in the subject, optional body explaining *why* (not *what*).
- Conventional prefix when useful: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `ci:`.
- One logical change per commit — no drive-by fixes mixed with feature work.
- **No AI author trailers.** AI-assisted commits land under the human author.
- **No `--amend` after push.** Create a new commit instead.

---

## PR Description

Mandatory, short, two sections only:

1. **Purpose** — why this change exists, the acceptance signal it unlocks.
2. **What changed** — readable prose on the substantive changes. Not a file list. Describe the logic shift, the new contract, the behaviour before/after.

No test-plan section (CI covers it). No emoji. Always include `Closes #<issue-number>`.

---

## No Thrash in the Diff

Before opening the PR, walk the diff sequentially:

- Any line added and then modified or removed → collapse to the final form.
- Any helper, class, method, component, or import introduced that's not used → delete.
- Any commented-out code → delete.

The diff a reviewer sees should be the *final* shape, not a history of your thinking.

---

## PR Review-Comment Resolution

A review comment is a **symptom signal**, not a narrow ticket. For every comment:

1. Fix the named issue.
2. Trace side effects on the other side — a backend rename → does the FE caller still match?
3. Trace side effects in the same file — stale docstrings and comments next to the fix.
4. Trace related tests and types — a bug fix usually means a test was weak, strengthen it.
5. Grep before replying — there is almost always more than one instance.
6. Reply to every comment once fixed — a one-line confirmation. Never leave a resolved comment silent.
7. Delete local branches after merge.

**Goal: one commit per review round, not three.**

---

## Pre-Submit Checklist

- Relevant test command green locally.
- Diff re-read against your stack's convention file.
- No-thrash walk done.
- Commit message + PR description match the rules above.
- Feature scope matches the approved plan — no silent expansion.
