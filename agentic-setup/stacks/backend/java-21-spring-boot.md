# Java 21 + Spring Boot 3.5.8 Backend Skill

Tech-stack conventions for Java 21 / Spring Boot 3.5.8 / Maven / JPA / REST.

---

## Stack Overview

- **Language:** Java 21
- **Framework:** Spring Boot 3.5.8
- **Build:** Maven 3.9+
- **REST:** Spring Web (path-versioned under `/api/v1/...`)
- **Persistence:** JPA / Hibernate
- **API Docs:** springdoc-openapi 2.8.4 (OpenAPI 3.x)
- **Testing:** JUnit 5 + Mockito
- **Observability:** Spring Boot 3.4+ native structured JSON logs (ECS format)

---

## Architecture: Light Hexagonal

- **Pure-Java domain model** (`<feature>/domain/model/`) — records, enums, value objects. Zero framework annotations.
- **Outbound ports** (`<feature>/domain/port/out/`) — services depend on repository/gateway interfaces, not Spring Data directly.
- **Adapters** (`<feature>/adapter/in/rest/` + `<feature>/adapter/out/persistence/`) — REST controllers, JPA repos, external integrations.
- **@Service** classes directly (no inbound port interfaces for MVP-scale).

**Package layout:**
```
backend/src/main/java/com/<company>/<project>/
├── Application.java
├── core/
│   ├── config/        # OpenAPI, CORS, Clock, etc. (cross-cutting)
│   └── web/           # Correlation ID filter, global exception handler
└── <feature>/
    ├── domain/
    │   ├── model/     # Pure Java records, enums, value objects
    │   ├── service/   # @Service business logic
    │   └── port/out/  # Outbound port interfaces
    └── adapter/
        ├── in/rest/   # @RestController + DTOs + OpenAPI annotations
        └── out/       # Port implementations (JPA, in-memory, etc.)
```

---

## REST API Rules

- **Path versioning:** `/api/v1/...`
- **Methods:** GET (read), POST (create), PATCH (partial update), DELETE (soft delete or removal)
- **Status codes:** 200 (OK), 201 (Created), 204 (No Content), 400 (Bad Request), 404 (Not Found), 409 (Conflict), 500 (Server Error)
- **Request validation:** `@Valid` on body; custom `@ControllerAdvice` / `GlobalExceptionHandler` for consistent error envelope
- **Error envelope:** `{ error: string, message: string, details?: object }`
- **OpenAPI annotations:** `@Operation`, `@ApiResponses`, `@Tag` on controller; `@Schema` on DTOs

**Example endpoint:**
```java
@RestController
@RequestMapping("/api/v1/items")
@Tag(name = "Items", description = "Item management")
public class ItemController {

  @PostMapping
  @Operation(summary = "Create item", description = "Creates a new item")
  @ApiResponses({
    @ApiResponse(responseCode = "201", description = "Item created"),
    @ApiResponse(responseCode = "400", description = "Invalid input")
  })
  public ResponseEntity<ItemResponse> create(@Valid @RequestBody CreateItemRequest req) {
    Item item = itemService.create(req.name(), req.description());
    return ResponseEntity.status(201).body(new ItemResponse(item));
  }
}
```

---

## Cursor-Based Pagination

For list endpoints, use cursor pagination ({items, nextCursor}):

```java
public record ItemListResponse(
  List<ItemDTO> items,
  String nextCursor  // null if no more results
) {}

// Service returns:
public ItemListResponse list(String cursor, int limit) {
  List<Item> items = repo.findAfterCursor(cursor, limit + 1);
  boolean hasMore = items.size() > limit;
  List<Item> page = hasMore ? items.subList(0, limit) : items;
  String next = hasMore ? encodeCursor(page.get(limit - 1).id()) : null;
  return new ItemListResponse(toDtos(page), next);
}
```

---

## Persistence

- **JPA repositories:** extend `JpaRepository<T, ID>`
- **DDL auto:**
  - `dev` profile: `ddl-auto=update` (auto-migrate schema)
  - `prod` profile: `ddl-auto=validate` (fail fast if schema mismatch)
  - `test` profile: managed by test containers or in-memory H2
- **No N+1 queries:** use `@EntityGraph` or explicit joins
- **Parameterized queries always:** never string-build filters from user input

---

## Caching

- **Redis:** Spring Cache abstraction with `@Cacheable`, `@CacheEvict`
- **Cache-busting:** on POST/PATCH/DELETE via `@CacheEvict`
- **TTL:** configure in `application.yaml` per cache name

---

## Observability

- **Structured logs:** Spring Boot 3.4+ native ECS format (JSON to stdout)
- **Correlation ID:** propagate via SLF4J MDC (pre-wired via `CorrelationIdFilter`)
- **Business events:** INFO-level log per state change with `event=<name>` + relevant IDs
- **Never log PII** beyond `user_id`; no passwords, API keys, tokens in logs

**Example:**
```java
log.info("item created", Map.of("event", "item.created", "item_id", item.id(), "user_id", req.userId()));
```

---

## Testing

- **Unit tests:** Pure domain logic, services via constructor injection (no Spring context)
- **Integration tests:** `@WebMvcTest` for controllers, `@DataJpaTest` for repos
- **BDD style:** Given/When/Then structure; test names are behaviour claims
- **No Thread.sleep:** use `@DirtiesContext` or test containers for isolation

**Example unit test:**
```java
@Test
void createItem_withValidInput_returnsItem() {
  // Given
  var repo = mock(ItemRepository.class);
  var service = new ItemService(repo);
  
  // When
  Item item = service.create("name", "desc");
  
  // Then
  assertThat(item.name()).isEqualTo("name");
  verify(repo).save(any());
}
```

---

## Build & Verify

```bash
mvn clean verify
```

Runs unit tests, integration tests (via `@SpringBootTest` or test containers), and generates OpenAPI spec.

---

## Dependencies (Locked)

```xml
<java.version>21</java.version>
<spring-boot.version>3.5.8</spring-boot.version>
<spring-cloud.version>2024.0.3</spring-cloud.version>
<springdoc-openapi.version>2.8.4</springdoc-openapi.version>
<junit.version>5.11.0</junit.version>
<mockito.version>5.12.0</mockito.version>
```

No security, no Flyway/Liquibase, no APM backend. Additive when feature demands.

---

## Common Pitfalls

- **`@Autowired` fields:** Never. Use constructor injection.
- **Too many constructor params:** If > 5 collaborators, missing abstraction.
- **String-built DB filters:** Always parameterized queries.
- **Global exception handler drift:** One central `GlobalExceptionHandler`, one error envelope shape.
- **Loose typing:** Wrap primitives with `record` value objects when rules accrue.

---

## See Also

- `docs/conventions/backend.md` — full rule set
- `docs/conventions/examples/backend-feature.md` — worked greeting feature
- `docs/conventions/security.md` — secrets, input validation
- `docs/conventions/testing.md` — test levels + patterns
