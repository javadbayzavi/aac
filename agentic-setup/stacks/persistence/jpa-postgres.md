# JPA + PostgreSQL Persistence Skill

Tech-stack conventions for Spring Data JPA / Hibernate / PostgreSQL.

---

## Stack Overview

- **ORM:** Hibernate 6.6+ (via Spring Data JPA)
- **Database:** PostgreSQL 16+
- **Connection:** HikariCP (default Spring Boot datasource)
- **Migrations:** None (use `ddl-auto` for MVP; add Flyway when schema history matters)
- **Cache:** Redis (optional, via Spring Cache + `@Cacheable`)

---

## Entity Design

- **Records preferred** for value objects (immutable, concise)
- **@Entity classes** for aggregate roots (mutable state, lifecycle)
- **Use @Embeddable** for composite value objects

**Entity example:**
```java
@Entity
@Table(name = "items")
public class Item {
  @Id
  @GeneratedValue(strategy = GenerationType.IDENTITY)
  private Long id;

  @Column(nullable = false)
  private String name;

  @Column(name = "created_at", nullable = false)
  private Instant createdAt;

  // Constructor, getters, domain methods
}
```

**Value object example:**
```java
@Embeddable
public record Money(
  BigDecimal amount,
  @Enumerated(EnumType.STRING) Currency currency
) {}
```

---

## Repository Pattern

- **Extend JpaRepository:** for CRUD + pagination
- **Custom queries:** use `@Query` with JPQL or native SQL (parameterized)
- **No N+1 queries:** use `@EntityGraph` for eager fetch joins

**Repository example:**
```java
@Repository
public interface ItemRepository extends JpaRepository<Item, Long> {

  @EntityGraph(attributePaths = {"category"})
  List<Item> findByNameContainingIgnoreCase(String name);

  @Query("SELECT i FROM Item i WHERE i.createdAt > :since ORDER BY i.id ASC LIMIT :limit")
  List<Item> findAfterCursor(@Param("since") Instant since, @Param("limit") int limit);
}
```

---

## Transactionality

- **@Transactional on service methods:** define boundaries explicitly
- **Default read-only=true** for queries (`@Transactional(readOnly=true)`)
- **Propagation=REQUIRED** (default) for state-changing ops

**Service example:**
```java
@Service
public class ItemService {

  @Transactional(readOnly = true)
  public Item getById(Long id) {
    return repo.findById(id).orElseThrow();
  }

  @Transactional
  public Item create(String name, String description) {
    Item item = new Item(name, description);
    return repo.save(item);
  }
}
```

---

## DDL Auto Behavior

**`dev` profile:**
```yaml
spring:
  jpa:
    hibernate:
      ddl-auto: update
```
Auto-creates/updates schema on startup. Quick iteration, local testing.

**`prod` profile:**
```yaml
spring:
  jpa:
    hibernate:
      ddl-auto: validate
```
Fails fast if schema doesn't match entities. No accidental schema drift.

**`test` profile:**
```yaml
spring:
  jpa:
    hibernate:
      ddl-auto: create-drop
```
Creates fresh schema for each test run (via Testcontainers or H2 in-memory).

---

## Cursor-Based Pagination

For scalable list endpoints, return {items, nextCursor} instead of offset-limit:

```java
public record ItemListResponse(
  List<ItemDTO> items,
  String nextCursor  // Base64-encoded ID of last item, null if no more
) {}

@Service
public class ItemService {

  @Transactional(readOnly = true)
  public ItemListResponse list(String cursor, int limit) {
    // Decode cursor (null on first request)
    Long afterId = cursor != null ? decodeCursor(cursor) : null;

    // Fetch limit+1 to detect if there are more results
    List<Item> results = repo.findByIdGreaterThanOrderById(afterId, limit + 1);

    boolean hasMore = results.size() > limit;
    List<Item> page = hasMore ? results.subList(0, limit) : results;

    String nextCursor = hasMore ? encodeCursor(page.get(limit - 1).getId()) : null;
    return new ItemListResponse(toDtos(page), nextCursor);
  }

  private String encodeCursor(Long id) {
    return Base64.getEncoder().encodeToString(id.toString().getBytes());
  }

  private Long decodeCursor(String cursor) {
    return Long.parseLong(new String(Base64.getDecoder().decode(cursor)));
  }
}
```

---

## Caching Strategy

Use Spring Cache abstraction with Redis backend:

```java
@Service
public class ItemService {

  @Cacheable(value = "items", key = "#id")
  public Item getById(Long id) {
    return repo.findById(id).orElseThrow();
  }

  @CacheEvict(value = "items", key = "#id")
  public Item update(Long id, String name) {
    Item item = getById(id);
    item.setName(name);
    return repo.save(item);
  }

  @CacheEvict(value = "items", allEntries = true)
  public Item create(String name) {
    return repo.save(new Item(name));
  }
}
```

**application.yaml:**
```yaml
spring:
  cache:
    type: redis
    redis:
      time-to-live: 3600000  # 1 hour
```

---

## Testing Persistence

- **Unit tests:** mock repository, test service logic
- **Integration tests:** use Testcontainers for PostgreSQL (real DB)
- **@DataJpaTest:** for repository tests only (no full app context)

**Repository test with Testcontainers:**
```java
@SpringBootTest
@Testcontainers
class ItemRepositoryTest {

  @Container
  static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>(DockerImageName.parse("postgres:16"));

  @Autowired
  ItemRepository repo;

  @DynamicPropertySource
  static void registerProperties(DynamicPropertyRegistry registry) {
    registry.add("spring.datasource.url", postgres::getJdbcUrl);
    registry.add("spring.datasource.username", postgres::getUsername);
    registry.add("spring.datasource.password", postgres::getPassword);
  }

  @Test
  void should_find_items_by_cursor() {
    // Given
    repo.saveAll(List.of(
      new Item("Item 1"),
      new Item("Item 2"),
      new Item("Item 3")
    ));

    // When
    List<Item> results = repo.findByIdGreaterThanOrderById(null, 3);

    // Then
    assertThat(results).hasSize(3);
  }
}
```

---

## No N+1 Pitfall

```java
// BAD: N+1 query (one per item.getCategory())
List<Item> items = repo.findAll();
items.forEach(item -> log.info(item.getCategory().getName()));

// GOOD: eager fetch with @EntityGraph
@EntityGraph(attributePaths = {"category"})
List<Item> findAll();
```

---

## Parameterized Queries Always

```java
// BAD: SQL injection risk
@Query("SELECT i FROM Item i WHERE i.name LIKE '%" + userInput + "%'")
List<Item> search(String userInput);

// GOOD: parameterized
@Query("SELECT i FROM Item i WHERE i.name LIKE CONCAT('%', :search, '%')")
List<Item> search(@Param("search") String userInput);
```

---

## See Also

- `docs/conventions/backend.md` — full rule set
- `docs/conventions/testing.md` — integration test patterns
- `docs/conventions/security.md` — no SQL injection, parameterized queries
