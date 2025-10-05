import { describe, it, expect, vi, beforeEach } from 'vitest';
import { api } from '../lib/api';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should make GET requests correctly', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResponse = { success: true, data: { id: '1', name: 'Test Server' } };
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockResponse));

    const result = await api('/servers');
    
    expect(invoke).toHaveBeenCalledWith('make_http_request', {
      url: expect.stringContaining('/servers'),
      method: 'GET',
      body: undefined
    });
    expect(result).toEqual(mockResponse.data);
  });

  it('should make POST requests correctly', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResponse = { success: true, data: { id: '1', name: 'New Server' } };
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockResponse));

    const serverData = { name: 'New Server', loader: 'vanilla', version: '1.20.1' };
    const result = await api('/servers', {
      method: 'POST',
      body: serverData
    });
    
    expect(invoke).toHaveBeenCalledWith('make_http_request', {
      url: expect.stringContaining('/servers'),
      method: 'POST',
      body: JSON.stringify(serverData)
    });
    expect(result).toEqual(mockResponse.data);
  });

  it('should handle API errors correctly', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockError = { success: false, error: 'Server not found' };
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockError));

    await expect(api('/servers/invalid-id')).rejects.toThrow();
  });

  it('should handle network errors correctly', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

    await expect(api('/servers')).rejects.toThrow('Network error');
  });
});
