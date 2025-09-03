# Guardian UI - Final Step & Polish Guide

## Overview
This guide provides complete instructions for finishing **Step 20: Polish & Tests** and finalizing the Guardian UI application. Follow these steps in order to complete the project.

## Current Status
- ✅ Steps 1-19: Complete
- 🔄 Step 20: Polish & Tests (In Progress)

## Step 20: Polish & Tests - Implementation Plan

### 20.1 Install Testing Dependencies
```bash
cd guardian-ui
npm install --save-dev vitest @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom
```

### 20.2 Configure Vitest
Create `vitest.config.ts`:
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
```

### 20.3 Create Test Setup
Create `src/test/setup.ts`:
```typescript
import '@testing-library/jest-dom'
import { beforeAll, afterEach, afterAll } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './mocks/server'

// Start server before all tests
beforeAll(() => server.listen())

// Reset handlers after each test
afterEach(() => {
  cleanup()
  server.resetHandlers()
})

// Clean up after all tests
afterAll(() => server.close())
```

### 20.4 Create MSW Test Server
Create `src/test/mocks/server.ts`:
```typescript
import { setupServer } from 'msw/node'
import { handlers } from '../../mocks/handlers'

export const server = setupServer(...handlers)
```

### 20.5 Update package.json Scripts
Add to `package.json`:
```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage",
    "test:run": "vitest run"
  }
}
```

### 20.6 Write Core Component Tests

#### Test: AppShell Component
Create `src/app/layout/__tests__/AppShell.test.tsx`:
```typescript
import { render, screen } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { AppShell } from '../AppShell'

const renderWithRouter = (component: React.ReactElement) => {
  return render(
    <BrowserRouter>
      {component}
    </BrowserRouter>
  )
}

describe('AppShell', () => {
  it('renders sidebar and main content area', () => {
    renderWithRouter(<AppShell />)
    
    expect(screen.getByRole('navigation')).toBeInTheDocument()
    expect(screen.getByRole('main')).toBeInTheDocument()
  })

  it('includes error boundary and toaster', () => {
    renderWithRouter(<AppShell />)
    
    // Error boundary should be present (wraps the app)
    // Toaster should be present
    expect(document.querySelector('[data-radix-toast-viewport]')).toBeInTheDocument()
  })
})
```

#### Test: StatusPill Component
Create `src/components/__tests__/StatusPill.test.tsx`:
```typescript
import { render, screen } from '@testing-library/react'
import { StatusPill } from '../StatusPill'

describe('StatusPill', () => {
  it('renders running status correctly', () => {
    render(<StatusPill status="running" />)
    
    expect(screen.getByText('Running')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toHaveClass('bg-green-500')
  })

  it('renders stopped status correctly', () => {
    render(<StatusPill status="stopped" />)
    
    expect(screen.getByText('Stopped')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toHaveClass('bg-red-500')
  })

  it('renders starting status with animation', () => {
    render(<StatusPill status="starting" />)
    
    expect(screen.getByText('Starting')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toHaveClass('animate-pulse')
  })
})
```

#### Test: StatCard Component
Create `src/components/__tests__/StatCard.test.tsx`:
```typescript
import { render, screen } from '@testing-library/react'
import { StatCard } from '../StatCard'
import { Activity } from 'lucide-react'

describe('StatCard', () => {
  it('renders title, value, and subtitle', () => {
    render(
      <StatCard
        title="TPS"
        value="20.0"
        subtitle="Ticks per second"
        icon={<Activity className="h-4 w-4" />}
      />
    )
    
    expect(screen.getByText('TPS')).toBeInTheDocument()
    expect(screen.getByText('20.0')).toBeInTheDocument()
    expect(screen.getByText('Ticks per second')).toBeInTheDocument()
  })

  it('renders trend indicator when provided', () => {
    render(
      <StatCard
        title="TPS"
        value="20.0"
        subtitle="Ticks per second"
        trend={{ value: 5, isPositive: true }}
      />
    )
    
    expect(screen.getByTestId('trend-indicator')).toBeInTheDocument()
  })
})
```

#### Test: EmptyState Component
Create `src/components/ui/__tests__/EmptyState.test.tsx`:
```typescript
import { render, screen, fireEvent } from '@testing-library/react'
import { EmptyState, NoServersEmptyState } from '../EmptyState'
import { Server } from 'lucide-react'

describe('EmptyState', () => {
  it('renders title and description', () => {
    render(
      <EmptyState
        title="No data found"
        description="Try refreshing the page"
      />
    )
    
    expect(screen.getByText('No data found')).toBeInTheDocument()
    expect(screen.getByText('Try refreshing the page')).toBeInTheDocument()
  })

  it('renders action button when provided', () => {
    const mockAction = jest.fn()
    
    render(
      <EmptyState
        title="No data found"
        action={{
          label: 'Refresh',
          onClick: mockAction
        }}
      />
    )
    
    const button = screen.getByText('Refresh')
    expect(button).toBeInTheDocument()
    
    fireEvent.click(button)
    expect(mockAction).toHaveBeenCalled()
  })

  it('renders custom icon when provided', () => {
    render(
      <EmptyState
        title="No servers"
        icon={<Server className="h-12 w-12" />}
      />
    )
    
    expect(screen.getByRole('img', { hidden: true })).toBeInTheDocument()
  })
})

describe('NoServersEmptyState', () => {
  it('renders with correct content and action', () => {
    const mockCreateServer = jest.fn()
    
    render(<NoServersEmptyState onCreateServer={mockCreateServer} />)
    
    expect(screen.getByText('No servers found')).toBeInTheDocument()
    expect(screen.getByText('Create Server')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Create Server'))
    expect(mockCreateServer).toHaveBeenCalled()
  })
})
```

#### Test: SkeletonLoader Component
Create `src/components/ui/__tests__/SkeletonLoader.test.tsx`:
```typescript
import { render, screen } from '@testing-library/react'
import { SkeletonLoader, ServerCardSkeleton } from '../SkeletonLoader'

describe('SkeletonLoader', () => {
  it('renders default skeleton', () => {
    render(<SkeletonLoader />)
    
    expect(screen.getByTestId('skeleton-loader')).toBeInTheDocument()
  })

  it('renders multiple skeletons when count is provided', () => {
    render(<SkeletonLoader count={3} />)
    
    const skeletons = screen.getAllByTestId('skeleton-loader')
    expect(skeletons).toHaveLength(3)
  })

  it('renders card variant correctly', () => {
    render(<SkeletonLoader variant="card" />)
    
    expect(screen.getByTestId('skeleton-loader')).toHaveClass('animate-pulse')
  })
})

describe('ServerCardSkeleton', () => {
  it('renders server card skeleton structure', () => {
    render(<ServerCardSkeleton />)
    
    expect(screen.getByTestId('server-card-skeleton')).toBeInTheDocument()
  })
})
```

#### Test: ErrorBoundary Component
Create `src/components/ui/__tests__/ErrorBoundary.test.tsx`:
```typescript
import { render, screen, fireEvent } from '@testing-library/react'
import { ErrorBoundary } from '../ErrorBoundary'

// Component that throws an error
const ThrowError = ({ shouldThrow }: { shouldThrow: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error')
  }
  return <div>No error</div>
}

describe('ErrorBoundary', () => {
  it('renders children when no error occurs', () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={false} />
      </ErrorBoundary>
    )
    
    expect(screen.getByText('No error')).toBeInTheDocument()
  })

  it('renders error UI when error occurs', () => {
    // Suppress console.error for this test
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {})
    
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    )
    
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    expect(screen.getByText('Try Again')).toBeInTheDocument()
    
    consoleSpy.mockRestore()
  })

  it('retries when retry button is clicked', () => {
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {})
    
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    )
    
    const retryButton = screen.getByText('Try Again')
    fireEvent.click(retryButton)
    
    // Should show the component again (no error)
    expect(screen.getByText('No error')).toBeInTheDocument()
    
    consoleSpy.mockRestore()
  })
})
```

### 20.7 Write Store Tests

#### Test: Servers Store
Create `src/store/__tests__/servers.test.ts`:
```typescript
import { renderHook, act } from '@testing-library/react'
import { useServersStore } from '../servers'
import { Server } from '@/lib/types'

const mockServer: Server = {
  id: '1',
  name: 'Test Server',
  status: 'running',
  loader: 'forge',
  version: '1.20.1',
  path: '/test/path',
  blueGreen: {
    active: 'blue',
    candidateHealthy: true
  }
}

describe('useServersStore', () => {
  beforeEach(() => {
    // Reset store state
    useServersStore.getState().setServers([])
    useServersStore.getState().setSelectedServer(null)
  })

  it('initializes with empty state', () => {
    const { result } = renderHook(() => useServersStore())
    
    expect(result.current.servers).toEqual([])
    expect(result.current.selectedServer).toBeNull()
  })

  it('adds server correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      result.current.addServer(mockServer)
    })
    
    expect(result.current.servers).toHaveLength(1)
    expect(result.current.servers[0]).toEqual(mockServer)
  })

  it('selects server correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      result.current.addServer(mockServer)
      result.current.setSelectedServer(mockServer.id)
    })
    
    expect(result.current.selectedServer).toEqual(mockServer)
  })

  it('updates server status correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      result.current.addServer(mockServer)
      result.current.updateServerStatus(mockServer.id, 'stopped')
    })
    
    expect(result.current.servers[0].status).toBe('stopped')
  })
})
```

### 20.8 Write Integration Tests

#### Test: Overview Page Integration
Create `src/app/pages/Servers/__tests__/Overview.test.tsx`:
```typescript
import { render, screen, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { Overview } from '../Overview'
import { useServersStore } from '@/store/servers'

// Mock the stores
jest.mock('@/store/servers')
jest.mock('@/store/realtime')

const mockUseServersStore = useServersStore as jest.MockedFunction<typeof useServersStore>

describe('Overview Page Integration', () => {
  beforeEach(() => {
    mockUseServersStore.mockReturnValue({
      selectedServer: {
        id: '1',
        name: 'Test Server',
        status: 'running',
        loader: 'forge',
        version: '1.20.1',
        path: '/test/path'
      },
      setSelectedServer: jest.fn(),
      updateServerStatus: jest.fn(),
      addServer: jest.fn(),
      removeServer: jest.fn(),
      promoteServer: jest.fn(),
      getServerById: jest.fn(),
      servers: []
    })
  })

  it('renders server overview when server is selected', async () => {
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    await waitFor(() => {
      expect(screen.getByText('Test Server')).toBeInTheDocument()
      expect(screen.getByText('Server Overview')).toBeInTheDocument()
    })
  })

  it('shows loading state initially', () => {
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    expect(screen.getByTestId('stats-grid-loading')).toBeInTheDocument()
  })

  it('shows empty state when no server is selected', () => {
    mockUseServersStore.mockReturnValue({
      selectedServer: null,
      setSelectedServer: jest.fn(),
      updateServerStatus: jest.fn(),
      addServer: jest.fn(),
      removeServer: jest.fn(),
      promoteServer: jest.fn(),
      getServerById: jest.fn(),
      servers: []
    })
    
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    expect(screen.getByText('No server selected')).toBeInTheDocument()
  })
})
```

### 20.9 Add Test IDs to Components

Update components to include test IDs:

#### Update StatusPill Component
Add to `src/components/StatusPill.tsx`:
```typescript
// Add data-testid to the main element
<div
  data-testid="status-pill"
  className={cn(
    "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
    statusClasses[status]
  )}
>
```

#### Update StatCard Component
Add to `src/components/StatCard.tsx`:
```typescript
// Add data-testid to the main element
<div
  data-testid="stat-card"
  className={cn("border rounded-lg p-4 space-y-2", className)}
>
  // ... existing content
  
  {trend && (
    <div data-testid="trend-indicator" className="flex items-center space-x-1">
      // ... trend content
    </div>
  )}
</div>
```

#### Update SkeletonLoader Component
Add to `src/components/ui/SkeletonLoader.tsx`:
```typescript
// Add data-testid to skeleton elements
<div
  data-testid="skeleton-loader"
  className={cn("animate-pulse", className)}
>
  {renderSkeleton()}
</div>

// Update ServerCardSkeleton
<div
  data-testid="server-card-skeleton"
  className="border rounded-lg p-4 space-y-3"
>
```

### 20.10 Performance Optimizations

#### Add React.memo to Components
Update key components with React.memo:

```typescript
// In StatusPill.tsx
export const StatusPill = React.memo<StatusPillProps>(({ status, className }) => {
  // ... component logic
})

// In StatCard.tsx
export const StatCard = React.memo<StatCardProps>(({ title, value, subtitle, icon, trend, className }) => {
  // ... component logic
})

// In EmptyState.tsx
export const EmptyState = React.memo<EmptyStateProps>(({ icon, title, description, action, secondaryAction, className, size = 'md' }) => {
  // ... component logic
})
```

#### Add useMemo and useCallback Optimizations
Update components with expensive calculations:

```typescript
// In PlayersTable.tsx
const filteredPlayers = useMemo(() => 
  players.filter(player =>
    player.name.toLowerCase().includes(searchQuery.toLowerCase())
  ), [players, searchQuery]
)

const handlePlayerAction = useCallback(async (action: string, player: Player, data?: any) => {
  // ... action logic
}, [serverId])
```

### 20.11 Accessibility Improvements

#### Add ARIA Labels and Roles
Update components with proper accessibility:

```typescript
// In StatusPill.tsx
<div
  data-testid="status-pill"
  role="status"
  aria-label={`Server status: ${status}`}
  className={cn(
    "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
    statusClasses[status]
  )}
>

// In StatCard.tsx
<div
  data-testid="stat-card"
  role="region"
  aria-labelledby={`stat-${title.toLowerCase()}`}
  className={cn("border rounded-lg p-4 space-y-2", className)}
>
  <h3 id={`stat-${title.toLowerCase()}`} className="text-sm font-medium text-muted-foreground">
    {title}
  </h3>
```

#### Add Keyboard Navigation
Ensure all interactive elements are keyboard accessible:

```typescript
// In EmptyState.tsx
<Button
  onClick={action.onClick}
  variant={action.variant || 'default'}
  size={size === 'sm' ? 'sm' : 'default'}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      action.onClick()
    }
  }}
>
  {action.label}
</Button>
```

### 20.12 Final Polish Tasks

#### 20.12.1 Code Quality
- [ ] Run ESLint and fix all warnings
- [ ] Run Prettier to format all code
- [ ] Add JSDoc comments to all exported functions
- [ ] Remove any unused imports or variables
- [ ] Ensure consistent naming conventions

#### 20.12.2 Documentation
- [ ] Update README.md with setup instructions
- [ ] Add component documentation with examples
- [ ] Document API integration points
- [ ] Add deployment instructions

#### 20.12.3 Error Handling
- [ ] Add error boundaries to all major sections
- [ ] Implement retry mechanisms for failed requests
- [ ] Add proper error logging
- [ ] Test error scenarios

#### 20.12.4 Performance
- [ ] Implement code splitting for routes
- [ ] Add lazy loading for heavy components
- [ ] Optimize bundle size
- [ ] Add performance monitoring

#### 20.12.5 Testing
- [ ] Achieve >80% test coverage
- [ ] Test all user interactions
- [ ] Test error scenarios
- [ ] Test accessibility features

### 20.13 Final Commands to Run

```bash
# Install dependencies
npm install

# Run tests
npm run test

# Run tests with coverage
npm run test:coverage

# Run linting
npm run lint

# Run type checking
npm run type-check

# Build for production
npm run build

# Start development server
npm run dev
```

### 20.14 Acceptance Criteria

The application is complete when:

- [ ] All tests pass with >80% coverage
- [ ] No ESLint warnings or errors
- [ ] All TypeScript types are properly defined
- [ ] All components are accessible (WCAG 2.1 AA)
- [ ] Error boundaries catch and handle errors gracefully
- [ ] Loading states provide good user feedback
- [ ] Empty states guide users to take action
- [ ] Toast notifications work for all user actions
- [ ] Real-time updates function correctly
- [ ] All server operations work with proper feedback
- [ ] Application builds without errors
- [ ] Performance is optimized
- [ ] Code is well-documented

### 20.15 Deployment Checklist

Before deploying:

- [ ] Update environment variables
- [ ] Configure production API endpoints
- [ ] Set up error monitoring (Sentry, etc.)
- [ ] Configure analytics
- [ ] Set up CI/CD pipeline
- [ ] Test in staging environment
- [ ] Performance test with realistic data
- [ ] Security audit
- [ ] Backup and rollback plan

## Quick Reference Commands

```bash
# Development
npm run dev

# Testing
npm run test
npm run test:ui
npm run test:coverage

# Building
npm run build
npm run preview

# Code Quality
npm run lint
npm run type-check
npm run format
```

## File Structure After Completion

```
guardian-ui/
├── src/
│   ├── app/
│   ├── components/
│   │   ├── __tests__/
│   │   └── ui/
│   │       └── __tests__/
│   ├── store/
│   │   └── __tests__/
│   ├── lib/
│   ├── test/
│   │   ├── setup.ts
│   │   └── mocks/
│   └── styles/
├── docs/
├── vitest.config.ts
├── package.json
└── README.md
```

This guide provides everything needed to complete the final step and polish the Guardian UI application to production quality.
