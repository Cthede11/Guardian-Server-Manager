import { render, screen, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { Overview } from '../Overview'
import { useServersStore } from '@/store/servers'
import { useRealtimeStore } from '@/store/realtime'

// Mock the stores
vi.mock('@/store/servers')
vi.mock('@/store/realtime')
vi.mock('@/components/ui/LoadingStates', () => ({
  useLoadingState: () => ({
    isLoading: false,
    error: null,
    startLoading: vi.fn(),
    stopLoading: vi.fn(),
    setLoadingError: vi.fn()
  })
}))

const mockUseServersStore = useServersStore as any
const mockUseRealtimeStore = useRealtimeStore as any

describe('Overview Page Integration', () => {
  beforeEach(() => {
    // Reset all mocks
    vi.clearAllMocks()
    
    mockUseServersStore.mockReturnValue({
      selectedServer: {
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
    })

    mockUseRealtimeStore.mockReturnValue({
      getServerData: vi.fn().mockReturnValue({
        metrics: {
          tps: 20.0,
          playersOnline: 5,
          playersMax: 20,
          heapUsed: 512,
          heapMax: 1024,
          tickP95: 50.0,
          tpsTrend: { value: 5, isPositive: true },
          memoryTrend: { value: 2, isPositive: false },
          tickTrend: { value: 1, isPositive: true }
        },
        health: {
          rcon: true,
          query: true,
          crashTickets: 0,
          freezeTickets: 0
        }
      })
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
    // Mock loading state to be true
    vi.doMock('@/components/ui/LoadingStates', () => ({
      useLoadingState: () => ({
        isLoading: true,
        error: null,
        startLoading: vi.fn(),
        stopLoading: vi.fn(),
        setLoadingError: vi.fn()
      })
    }))
    
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    expect(screen.getByTestId('stats-grid-loading')).toBeInTheDocument()
  })

  it('shows empty state when no server is selected', () => {
    mockUseServersStore.mockReturnValue({
      selectedServer: null
    })
    
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    expect(screen.getByText('No server selected')).toBeInTheDocument()
  })

  it('displays server metrics correctly', async () => {
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    await waitFor(() => {
      expect(screen.getByText('20.0')).toBeInTheDocument() // TPS
      expect(screen.getByText('5')).toBeInTheDocument() // Players online
      expect(screen.getByText('512MB')).toBeInTheDocument() // Memory
      expect(screen.getByText('50.0ms')).toBeInTheDocument() // Tick time
    })
  })

  it('displays health status correctly', async () => {
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    await waitFor(() => {
      expect(screen.getByText('Health Status')).toBeInTheDocument()
      expect(screen.getByText('RCON')).toBeInTheDocument()
      expect(screen.getByText('Query')).toBeInTheDocument()
      expect(screen.getByText('Crashes')).toBeInTheDocument()
      expect(screen.getByText('Freezes')).toBeInTheDocument()
    })
  })

  it('displays blue/green deployment status', async () => {
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    await waitFor(() => {
      expect(screen.getByText('Blue/Green Deployment')).toBeInTheDocument()
      expect(screen.getByText('Blue')).toBeInTheDocument()
      expect(screen.getByText('Green')).toBeInTheDocument()
      expect(screen.getByText('Active:')).toBeInTheDocument()
      expect(screen.getByText('blue')).toBeInTheDocument()
      expect(screen.getByText('Healthy')).toBeInTheDocument()
    })
  })

  it('shows error state when there is an error', () => {
    mockUseServersStore.mockReturnValue({
      selectedServer: {
        id: '1',
        name: 'Test Server',
        status: 'running'
      }
    })

    // Mock the loading state to have an error
    vi.mock('@/components/ui/LoadingStates', () => ({
      useLoadingState: () => ({
        isLoading: false,
        error: new Error('Test error'),
        startLoading: vi.fn(),
        stopLoading: vi.fn(),
        setLoadingError: vi.fn()
      })
    }))
    
    render(
      <BrowserRouter>
        <Overview />
      </BrowserRouter>
    )
    
    expect(screen.getByText('Failed to load server data')).toBeInTheDocument()
    expect(screen.getByText('Test error')).toBeInTheDocument()
  })
})
