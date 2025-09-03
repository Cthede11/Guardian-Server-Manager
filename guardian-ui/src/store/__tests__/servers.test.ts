import { renderHook, act } from '@testing-library/react'
import { useServersStore } from '../servers'
import { ServerSummary } from '@/lib/types'

const mockServer: ServerSummary = {
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

// Mock the API
vi.mock('@/lib/api', () => ({
  api: {
    getServers: vi.fn().mockResolvedValue({ ok: true, data: [] }),
    createServer: vi.fn().mockResolvedValue({ ok: true, data: null }),
    serverAction: vi.fn().mockResolvedValue({ ok: true }),
    getServerHealth: vi.fn().mockResolvedValue({ ok: true, data: null }),
    getServerSettings: vi.fn().mockResolvedValue({ ok: true, data: null }),
    updateServerSettings: vi.fn().mockResolvedValue({ ok: true, data: null }),
  }
}))

describe('useServersStore', () => {
  beforeEach(() => {
    // Reset store state
    useServersStore.setState({
      servers: [],
      selectedServerId: null,
      serverHealth: {},
      serverSettings: {},
      loading: false,
      error: null,
    })
  })

  it('initializes with empty state', () => {
    const { result } = renderHook(() => useServersStore())
    
    expect(result.current.servers).toEqual([])
    expect(result.current.selectedServerId).toBeNull()
    expect(result.current.loading).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('selects server correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      result.current.selectServer('1')
    })
    
    expect(result.current.selectedServerId).toBe('1')
  })

  it('gets selected server correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      useServersStore.setState({ 
        servers: [mockServer],
        selectedServerId: '1'
      })
    })
    
    const selectedServer = result.current.getSelectedServer()
    expect(selectedServer).toEqual(mockServer)
  })

  it('gets server by id correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      useServersStore.setState({ 
        servers: [mockServer]
      })
    })
    
    const server = result.current.getServerById('1')
    expect(server).toEqual(mockServer)
    
    const nonExistentServer = result.current.getServerById('2')
    expect(nonExistentServer).toBeNull()
  })

  it('clears error correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      useServersStore.setState({ error: 'Test error' })
    })
    
    expect(result.current.error).toBe('Test error')
    
    act(() => {
      result.current.clearError()
    })
    
    expect(result.current.error).toBeNull()
  })

  it('handles server status updates correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      useServersStore.setState({ 
        servers: [mockServer]
      })
    })
    
    // Test start server
    act(() => {
      result.current.startServer('1')
    })
    
    expect(result.current.servers[0].status).toBe('starting')
    
    // Test stop server
    act(() => {
      result.current.stopServer('1')
    })
    
    expect(result.current.servers[0].status).toBe('stopping')
  })

  it('handles server promotion correctly', () => {
    const { result } = renderHook(() => useServersStore())
    
    act(() => {
      useServersStore.setState({ 
        servers: [mockServer]
      })
    })
    
    act(() => {
      result.current.promoteServer('1')
    })
    
    expect(result.current.servers[0].blueGreen.active).toBe('green')
  })
})
