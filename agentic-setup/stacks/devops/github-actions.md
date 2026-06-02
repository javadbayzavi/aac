# GitHub Actions CI/CD Skill

Tech-stack conventions for GitHub Actions workflows, branch protection, secrets, and submission.

---

## Workflow Overview

Two workflows for agentic development challenges:

1. **per-push.yml** — runs on push to any branch (except main) + PR to main
   - Parallel backend (`mvn clean verify`) and frontend (`npm lint / typecheck / test / build`) jobs
   - Fast feedback loop (5-10 min)
   - Required gate before merge

2. **merge-to-main.yml** — runs on push to main (after merge)
   - Builds production artifacts (backend jar, frontend bundle)
   - Uploads as workflow artifacts for review download
   - Optional: deployment pipeline (out of scope for MVP)

---

## per-push.yml Example

```yaml
name: Per-Push Verification

on:
  push:
    branches-ignore: [main]
  pull_request:
    branches: [main]

jobs:
  backend:
    name: Backend (Java 21 / Spring Boot)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Java 21
        uses: actions/setup-java@v4
        with:
          java-version: '21'
          distribution: 'temurin'

      - name: Cache Maven dependencies
        uses: actions/cache@v4
        with:
          path: ~/.m2/repository
          key: ${{ runner.os }}-maven-${{ hashFiles('backend/pom.xml') }}
          restore-keys: |
            ${{ runner.os }}-maven-

      - name: Run tests
        run: cd backend && mvn clean verify

  frontend:
    name: Frontend (Angular 21)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        run: cd frontend && npm ci

      - name: Lint
        run: cd frontend && npm run lint

      - name: Type check
        run: cd frontend && npm run typecheck

      - name: Test
        run: cd frontend && npm test

      - name: Build
        run: cd frontend && npm run build
```

---

## merge-to-main.yml Example

```yaml
name: Merge to Main

on:
  push:
    branches: [main]

jobs:
  build-and-upload:
    name: Build Production Artifacts
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Java 21
        uses: actions/setup-java@v4
        with:
          java-version: '21'
          distribution: 'temurin'

      - name: Build backend jar
        run: cd backend && mvn clean package -DskipTests

      - name: Set up Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Build frontend bundle
        run: cd frontend && npm ci && npm run build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: production-build
          path: |
            backend/target/*.jar
            frontend/dist/
          retention-days: 1
```

---

## Workflow Best Practices

**Action versions:** Always pin to major version (`@v4`), never `@latest` or `@main`:
```yaml
- uses: actions/checkout@v4      # ✓ Good
- uses: actions/checkout@latest  # ✗ Bad
```

**Concurrency groups:** Prevent multiple runs of same workflow on a branch:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Parallel jobs:** Run independent jobs in parallel (e.g., backend + frontend):
```yaml
jobs:
  backend:
    runs-on: ubuntu-latest
    ...
  frontend:
    runs-on: ubuntu-latest
    ...
    # Both run in parallel, no explicit `needs:`
```

**Sequential jobs:** Only when there's a data dependency:
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    ...
  deploy:
    runs-on: ubuntu-latest
    needs: build  # Deploy after build completes
    ...
```

---

## Secrets Handling

**Never hardcode credentials in YAML.** Use GitHub Secrets:

```yaml
env:
  DB_PASSWORD: ${{ secrets.DB_PASSWORD }}

steps:
  - name: Run migrations
    run: |
      export DATABASE_URL=postgresql://user:${{ secrets.DB_PASSWORD }}@localhost/db
      ./migrations.sh
```

**Masking sensitive output:**
```yaml
- name: Fetch API key
  run: |
    KEY=$(curl -s https://api.example.com/key)
    echo "::add-mask::$KEY"
    echo "KEY=$KEY" >> $GITHUB_ENV
```

**No PII in logs:** Never echo passwords, tokens, or API keys (even with masking).

---

## Branch Protection

Protect `main` with required checks:

1. **Settings → Branches → Branch protection rules** for `main`
2. **Require status checks to pass before merging**
   - Select per-push.yml workflows as required checks
3. **Dismiss stale pull request approvals when new commits are pushed**
4. **Require code review from one reviewer** (per your team policy)
5. **Allow force pushes: never** (prevent accidental overwrites)

---

## Submission Prep

End of challenge:

1. Ensure private GitHub repo visibility is correct
2. Add collaborator `coding-challenge@getworkflex.com`
3. Push all code to `main`
4. Confirm `main` builds green on final push (check per-push.yml + merge-to-main.yml)
5. Provide repo link to Workflex recruiter

---

## Cache Strategy

Cache dependencies to speed up builds:

```yaml
- name: Cache Maven dependencies
  uses: actions/cache@v4
  with:
    path: ~/.m2/repository
    key: ${{ runner.os }}-maven-${{ hashFiles('backend/pom.xml') }}
    restore-keys: |
      ${{ runner.os }}-maven-
```

Invalidates cache when `pom.xml` changes; restores on miss.

---

## Common Pitfalls

- **No action version pinning:** always major version (`@v4`), never floating
- **Global concurrency group:** use branch-aware groups to prevent cross-branch interference
- **Secrets in logs:** never echo API keys, even with masking; log only IDs
- **No branch protection on main:** always require CI checks before merge
- **Long artifact retention:** set `retention-days: 1` to avoid storage bloat

---

## See Also

- `docs/conventions/devops.md` — full rule set
- `docs/conventions/security.md` — secrets handling, GitHub Actions best practices
