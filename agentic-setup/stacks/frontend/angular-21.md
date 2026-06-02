# Angular 21 + TypeScript Frontend Skill

Tech-stack conventions for Angular 21 / TypeScript / Jest / standalone components / signals.

---

## Stack Overview

- **Framework:** Angular 21.2.8
- **Language:** TypeScript 5.9 (strict mode)
- **Build:** `npm` / `ng`
- **Testing:** Jest 30 + jest-preset-angular 16
- **Styling:** SCSS (BEM naming)
- **HTTP:** `HttpClient` + functional interceptors
- **State:** Angular signals + services (no external state library by default)
- **Routing:** Standalone components + lazy loading

---

## Component Architecture

- **Standalone components:** `standalone: true`, no NgModules (except root)
- **Signals:** `signal()` for reactive state, `computed()` for derived state, `effect()` for side effects
- **Change detection:** `OnPush` by default; minimize rendering scope
- **Dependency injection:** `inject()` pattern, not constructor params

**Component example:**
```typescript
import { Component, inject, signal, effect } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

@Component({
  selector: 'app-items',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `...`
})
export class ItemsComponent {
  private http = inject(HttpClient);
  items = signal<Item[]>([]);
  loading = signal(false);

  constructor() {
    effect(() => {
      this.loadItems();
    });
  }

  private loadItems() {
    this.loading.set(true);
    this.http.get<Item[]>('/api/v1/items')
      .pipe(takeUntilDestroyed())
      .subscribe(items => {
        this.items.set(items);
        this.loading.set(false);
      });
  }
}
```

---

## HTTP + Interceptors

- **Services own HttpClient:** never direct `HttpClient` in components
- **Functional interceptors:** `HTTP_INTERCEPTORS` provider with `Injectable` factory functions
- **Error handling:** service-level with error state (not throwing from interceptor)

**HTTP service example:**
```typescript
@Injectable({ providedIn: 'root' })
export class ItemService {
  private http = inject(HttpClient);

  getItems(cursor?: string): Observable<ItemListResponse> {
    const params = new HttpParams();
    if (cursor) params = params.set('cursor', cursor);
    return this.http.get<ItemListResponse>('/api/v1/items', { params });
  }
}
```

**Functional interceptor:**
```typescript
export function correlationIdInterceptor(
  req: HttpRequest<unknown>,
  next: HttpHandlerFn
): Observable<HttpEvent<unknown>> {
  const correlationId = inject(CorrelationIdService).getId();
  const cloned = req.clone({
    headers: req.headers.set('X-Correlation-Id', correlationId)
  });
  return next(cloned);
}

// In app.config.ts:
export const appConfig: ApplicationConfig = {
  providers: [
    provideHttpClient(
      withInterceptors([correlationIdInterceptor])
    )
  ]
};
```

---

## State Management

- **Default:** signals + services (no Redux / NgRx unless feature plan calls for it)
- **Discriminated unions:** for multi-state components (idle / loading / ready / error)

**State example:**
```typescript
type ItemState = 
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'ready'; items: Item[]; nextCursor: string | null }
  | { status: 'error'; message: string };

@Injectable({ providedIn: 'root' })
export class ItemStore {
  state = signal<ItemState>({ status: 'idle' });

  loadItems(cursor?: string) {
    this.state.set({ status: 'loading' });
    this.itemService.getItems(cursor).subscribe(
      res => this.state.set({ status: 'ready', items: res.items, nextCursor: res.nextCursor }),
      err => this.state.set({ status: 'error', message: err.message })
    );
  }
}

// In template:
@switch (store.state().status) {
  @case ('loading') { <p>Loading...</p> }
  @case ('ready') { <app-items [items]="store.state().items" /> }
  @case ('error') { <p>Error: {{ store.state().message }}</p> }
}
```

---

## Reactive Forms

- `FormBuilder` for complex forms
- `Validators` (built-in) + custom validators
- `markAllAsTouched()` for validation display on submit
- Track form state (pristine, dirty, valid, touched)

**Form example:**
```typescript
form = this.fb.group({
  name: ['', [Validators.required, Validators.minLength(2)]],
  email: ['', [Validators.required, Validators.email]],
  description: ['']
});

onSubmit() {
  if (this.form.invalid) {
    this.form.markAllAsTouched();
    return;
  }
  this.itemService.create(this.form.value).subscribe(...);
}
```

---

## Routing + Guards

- **Standalone components:** use `loadComponent` for lazy loading
- **Functional guards:** `inject()` dependencies, return `Observable<boolean>` or redirect

**Routes example:**
```typescript
export const routes: Routes = [
  {
    path: 'items',
    loadComponent: () => import('./items/items.component').then(m => m.ItemsComponent),
    canActivate: [authGuard]
  }
];

// Functional guard:
export const authGuard: CanActivateFn = (route, state) => {
  const authService = inject(AuthService);
  return authService.isAuthenticated()
    ? true
    : inject(Router).createUrlTree(['/login']);
};
```

---

## Testing

- **Jest 30 + jest-preset-angular 16**
- **Unit tests:** services, pipes, pure helpers (via `HttpTestingController` for HTTP)
- **Component tests:** conditional rendering, form validation, user-input flows
- **Skip pure presentational components** (no test needed if just `@Input` → template)
- **BDD naming:** describe what the component/service does, not the test mechanics

**Service test:**
```typescript
describe('ItemService', () => {
  let service: ItemService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [ItemService]
    });
    service = TestBed.inject(ItemService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  it('should load items from /api/v1/items', () => {
    service.getItems().subscribe(res => {
      expect(res.items.length).toBe(2);
    });

    const req = httpMock.expectOne('/api/v1/items');
    expect(req.request.method).toBe('GET');
    req.flush({ items: [{id: 1, name: 'Item 1'}, {id: 2, name: 'Item 2'}], nextCursor: null });
  });

  afterEach(() => httpMock.verify());
});
```

**Component test:**
```typescript
describe('ItemsComponent', () => {
  it('should display loading state while fetching', () => {
    const itemService = jasmine.createSpyObj('ItemService', ['getItems']);
    itemService.getItems.and.returnValue(of({items: [], nextCursor: null}));

    const { component, fixture } = render(ItemsComponent, {
      providers: [{ provide: ItemService, useValue: itemService }]
    });

    expect(component.loading()).toBe(false);
    expect(fixture.debugElement.query(By.css('p[data-testid="loading"]'))).toBeFalsy();
  });
});
```

---

## File Layout

```
src/
├── app/
│   ├── app.component.ts / spec.ts
│   ├── app.config.ts
│   ├── app.routes.ts
│   ├── core/
│   │   ├── interceptors/        # Functional HTTP interceptors
│   │   ├── guards/              # Functional route guards
│   │   └── services/            # Cross-feature services (auth, correlation ID, etc.)
│   ├── shared/
│   │   ├── components/          # Reusable presentational components
│   │   ├── pipes/               # Custom pipes
│   │   └── utils/               # Pure helpers
│   └── features/
│       └── <feature>/
│           ├── components/      # Feature-specific components
│           ├── services/        # Feature services + stores
│           └── *.spec.ts        # Tests co-located with source
├── environments/
│   ├── environment.ts           # dev
│   └── environment.prod.ts
└── assets/
```

---

## Code Style

- **TypeScript strict:** `strict: true` in tsconfig
- **No `any`:** use generics or union types; if unavoidable, comment `// FIXME: why?`
- **Named exports:** default exports only for lazy-loaded route components
- **Prettier:** `npm run format:check` before commit
- **Import order:** Angular → rxjs → local services → local components

---

## Common Pitfalls

- **API calls in template:** Never. Calls live in services, triggered from `ngOnInit`, `effect()`, or user events.
- **Manual subscribe without cleanup:** Always `takeUntilDestroyed()` or use `async` pipe
- **Global state at app root:** Hoist only to narrowest common ancestor
- **No data-testid selectors:** use `data-testid` for reliable test selectors, not classes or structure
- **Snapshot tests without buy-in:** avoid unless explicitly approved

---

## See Also

- `docs/conventions/frontend.md` — full rule set
- `docs/conventions/examples/frontend-feature.md` — worked greeting component
- `docs/conventions/security.md` — input validation, no secrets
- `docs/conventions/testing.md` — test levels + patterns
