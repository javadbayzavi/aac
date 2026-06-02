# Security: Cross-Cutting Conventions

Tech-stack agnostic security rules for all code.

---

## Secrets Handling

**Never hardcode:**
- API keys, tokens, passwords in source code
- Database credentials in application.yaml
- Private keys, certificates in the repo

**Always use:**
- Environment variables (prefixed, documented in `.env.example`)
- GitHub Secrets for CI/CD
- `.gitignore` entries: `.env`, `*.key`, `*.pem`, `secrets.yaml`

**Example:**
```bash
# .env.example
DATABASE_URL=postgresql://user:password@localhost/db
API_KEY=your_api_key_here

# .gitignore
.env
*.key
*.pem
```

---

## Input Validation

**Backend:**
- `@Valid` on all request bodies
- Custom validators for domain rules
- Parameterized queries always (no string concatenation for DB filters)
- Whitelist input patterns; reject on mismatch

**Frontend:**
- Form validation (required, min/max length, email, etc.)
- Trim user input (no leading/trailing whitespace)
- Reject before submission if invalid
- Server-side validation is authoritative (never trust client)

**Example:**
```java
@PostMapping
public ResponseEntity<?> create(@Valid @RequestBody CreateRequest req) {
  // @Valid ensures req.name is not null, meets @Size constraints
  // Never trust client input; re-validate in domain
  if (!isValidName(req.name())) {
    return ResponseEntity.badRequest().body("Invalid name");
  }
  return ...;
}
```

---

## SQL Injection Prevention

**Never do this:**
```java
// BAD: SQL injection vector
String query = "SELECT * FROM items WHERE name LIKE '%" + userInput + "%'";
```

**Always parameterize:**
```java
// GOOD: parameterized query
@Query("SELECT i FROM Item i WHERE i.name LIKE CONCAT('%', :search, '%')")
List<Item> search(@Param("search") String userInput);

// Or via framework:
List<Item> findByNameContainingIgnoreCase(String userInput);
```

---

## No PII in Logs

**Log only:**
- `user_id` (opaque identifier, not email)
- `request_id` / `correlation_id`
- Business event names
- Error messages (no stack traces in prod)

**Never log:**
- Email addresses, phone numbers, names
- Passwords, tokens, API keys
- Credit card / SSN / government ID numbers
- Full request/response bodies
- Stack traces in production

**Example:**
```java
// BAD
log.info("User created: " + user);  // logs email, password hash, etc.

// GOOD
log.info("user created", Map.of("event", "user.created", "user_id", user.id()));
```

---

## CORS Configuration

**Default: reject cross-origin requests unless explicitly allowed.**

```yaml
# application.yaml
spring:
  web:
    cors:
      allowed-origins: http://localhost:4200, https://app.example.com
      allowed-methods: GET, POST, PATCH, DELETE
      allowed-headers: Content-Type, Authorization, X-Correlation-Id
      allow-credentials: true
      max-age: 3600
```

**Frontend:**
- Send credentials in requests only to trusted origins
- Never expose sensitive headers in responses

---

## Authentication & Authorization

**When feature plan requires it:**
- Use Spring Security (minimal starter, not full suite)
- Stateless (JWT or session + CSRF token)
- Hash passwords with bcrypt (never plaintext, never MD5)
- Rate-limit login attempts (prevent brute force)
- Rotate session tokens after successful login

**Not in MVP scope unless feature explicitly requires.**

---

## HTTP Headers

**Always set:**
```java
// In CORS config or global filter
response.setHeader("X-Content-Type-Options", "nosniff");
response.setHeader("X-Frame-Options", "DENY");
response.setHeader("X-XSS-Protection", "1; mode=block");
response.setHeader("Strict-Transport-Security", "max-age=31536000");
```

---

## Frontend Input Sanitization

**Never:**
```typescript
// BAD: XSS vector
element.innerHTML = userInput;
element.innerHTML = `<p>${userInput}</p>`;
```

**Always:**
```typescript
// GOOD: Angular interpolation (auto-escapes)
<p>{{ userInput }}</p>

// GOOD: DomSanitizer for trusted HTML
<div [innerHTML]="domSanitizer.sanitize(SecurityContext.HTML, userInput)"></div>
```

---

## Rate Limiting

**Not in MVP scope** unless feature plan calls for it. When added:
- Use Spring Cloud Gateway or servlet filter
- Limit by IP + user ID
- Return 429 (Too Many Requests) on excess

---

## Dependency Scanning

**No CVE vulnerabilities in production dependencies.**

```bash
# Check for known vulnerabilities
mvn dependency-check:check
npm audit
```

If found: update dependency immediately, or document exception + mitigation.

---

## Code Review Checklist

Every PR review: ✓ No hardcoded secrets, ✓ No SQL injection vectors, ✓ Input validated, ✓ No PII in logs, ✓ HTTPS enforced (if applicable), ✓ No new CVEs introduced.

---

## See Also

- `docs/conventions/security.md` — full rule set (project-specific)
- OWASP Top 10
- CWE/SANS Top 25
