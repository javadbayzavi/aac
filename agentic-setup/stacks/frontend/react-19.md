# React 19 + TypeScript Frontend Skill

Tech-stack conventions for React 19 / TypeScript / Vitest / hooks-based separation of concerns / BDD testing.

---

## Stack Overview

- **Framework:** React 19
- **Language:** TypeScript 5.x (strict mode)
- **Build:** Vite
- **Testing:** Vitest + React Testing Library
- **Styling:** CSS Modules or Tailwind (project-configured)
- **HTTP:** custom hooks wrapping `fetch` or `axios` — never raw calls in components
- **State:** `useState` / `useReducer` / Context (no external state library by default)
- **Routing:** React Router v7

---

## Separation of Concerns: UI / Service / API

The core rule: **components render, hooks orchestrate, services/APIs fetch.**

```
Component          → renders UI, calls domain hooks only
Domain hook        → owns business logic, state, error handling
API hook / service → owns HTTP calls, maps responses to domain types
```

### Built-in hooks (defaults)
Use React built-ins directly inside domain hooks:
- `useState` — local UI state
- `useReducer` — complex multi-state transitions
- `useEffect` — side effects (prefer custom hooks over raw useEffect in components)
- `useCallback` / `useMemo` — stabilise references passed as props

### Custom hooks: two layers

**API hooks** — thin wrappers around HTTP. One function per endpoint. No business logic.
```typescript
// hooks/api/useItemsApi.ts
export function useItemsApi() {
  const get = async (cursor?: string): Promise<ItemListResponse> => {
    const res = await fetch(`/api/v1/items${cursor ? `?cursor=${cursor}` : ''}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  };
  return { get };
}
```

**Domain hooks** — own business state and orchestrate API hooks. No JSX, no direct fetch.
```typescript
// hooks/domain/useItems.ts
import { useState, useCallback } from 'react';
import { useItemsApi } from '../api/useItemsApi';
import type { Item } from '../../types/item';

type ItemState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'ready'; items: Item[]; nextCursor: string | null }
  | { status: 'error'; message: string };

export function useItems() {
  const api = useItemsApi();
  const [state, setState] = useState<ItemState>({ status: 'idle' });

  const load = useCallback(async (cursor?: string) => {
    setState({ status: 'loading' });
    try {
      const res = await api.get(cursor);
      setState({ status: 'ready', items: res.items, nextCursor: res.nextCursor });
    } catch (e) {
      setState({ status: 'error', message: (e as Error).message });
    }
  }, [api]);

  return { state, load };
}
```

**Component** — calls domain hook only. No API calls, no business logic.
```typescript
// features/items/ItemsPage.tsx
import { useEffect } from 'react';
import { useItems } from '../../hooks/domain/useItems';

export function ItemsPage() {
  const { state, load } = useItems();

  useEffect(() => { load(); }, [load]);

  if (state.status === 'loading') return <p>Loading…</p>;
  if (state.status === 'error') return <p>Error: {state.message}</p>;
  if (state.status === 'idle') return null;

  return (
    <ul>
      {state.items.map(item => <li key={item.id}>{item.name}</li>)}
    </ul>
  );
}
```

---

## File Layout

```
src/
├── hooks/
│   ├── api/              # One file per API resource — HTTP only
│   └── domain/           # One file per business concept — logic + state
├── features/
│   └── <feature>/
│       ├── <Feature>Page.tsx       # Route-level component
│       ├── <Feature>Form.tsx       # Form sub-component
│       └── <Feature>.test.tsx      # Co-located tests
├── shared/
│   ├── components/       # Reusable presentational components
│   └── utils/            # Pure helpers
├── types/                # Domain types — no framework imports
├── App.tsx
└── main.tsx
```

---

## Testing: Vitest + React Testing Library + BDD

- **BDD naming:** `describe` = the hook or component, `it` = a behaviour claim starting with "should"
- **Test domain hooks:** mock the API hook layer only — test business logic in isolation
- **Test components:** render with a mocked domain hook — test what the user sees
- **Test API hooks:** use `msw` (Mock Service Worker) to intercept HTTP — never mock `fetch` directly
- **Skip:** implementation details, internal state shape, snapshot tests

**Domain hook test:**
```typescript
// hooks/domain/useItems.test.ts
import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useItems } from './useItems';
import * as apiModule from '../api/useItemsApi';

describe('useItems', () => {
  it('should transition to ready state with items after successful load', async () => {
    vi.spyOn(apiModule, 'useItemsApi').mockReturnValue({
      get: async () => ({ items: [{ id: '1', name: 'Widget' }], nextCursor: null })
    });

    const { result } = renderHook(() => useItems());
    expect(result.current.state.status).toBe('idle');

    await act(async () => { await result.current.load(); });

    expect(result.current.state.status).toBe('ready');
    if (result.current.state.status === 'ready') {
      expect(result.current.state.items[0].name).toBe('Widget');
    }
  });

  it('should transition to error state when API fails', async () => {
    vi.spyOn(apiModule, 'useItemsApi').mockReturnValue({
      get: async () => { throw new Error('Network error'); }
    });

    const { result } = renderHook(() => useItems());
    await act(async () => { await result.current.load(); });

    expect(result.current.state.status).toBe('error');
  });
});
```

**Component test:**
```typescript
// features/items/ItemsPage.test.tsx
import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import { ItemsPage } from './ItemsPage';
import * as domainModule from '../../hooks/domain/useItems';

describe('ItemsPage', () => {
  it('should display items when state is ready', () => {
    vi.spyOn(domainModule, 'useItems').mockReturnValue({
      state: { status: 'ready', items: [{ id: '1', name: 'Widget' }], nextCursor: null },
      load: vi.fn()
    });

    render(<ItemsPage />);
    expect(screen.getByText('Widget')).toBeInTheDocument();
  });

  it('should display loading indicator while fetching', () => {
    vi.spyOn(domainModule, 'useItems').mockReturnValue({
      state: { status: 'loading' },
      load: vi.fn()
    });

    render(<ItemsPage />);
    expect(screen.getByText('Loading…')).toBeInTheDocument();
  });
});
```

---

## Code Style

- **TypeScript strict:** `strict: true` in tsconfig
- **No `any`:** use generics or discriminated unions
- **Named exports:** no default exports except route-level page components
- **No inline styles:** CSS Modules or Tailwind classes only
- **`data-testid`:** use for test selectors, not class names or structure

---

## Common Pitfalls

- **fetch in components:** Never. API calls live in API hooks only.
- **Business logic in components:** Never. Domain hooks own logic and state.
- **`useEffect` in components for data fetching:** use a domain hook instead — components call `load()` explicitly.
- **Mocking `fetch` directly in tests:** use `msw` for API hooks, mock the API hook for domain hook tests.
- **Missing discriminated union exhaustion:** always handle all status branches in components.

---

## See Also

- `agentic-setup/stacks/security/cross-cutting.md` — input validation, no secrets in frontend
- `agentic-setup/stacks/devops/pr-workflow.md` — branch, commit, PR conventions
