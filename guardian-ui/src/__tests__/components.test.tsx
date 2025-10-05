import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ServerList } from '../components/ServerList';
import { CreateServerModal } from '../components/CreateServerModal';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock API
vi.mock('../lib/api', () => ({
  api: vi.fn(),
}));

const MockedRouter = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>{children}</BrowserRouter>
);

describe('ServerList Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render server list correctly', async () => {
    const { api } = await import('../lib/api');
    const mockServers = [
      { id: '1', name: 'Test Server 1', status: 'running', tps: 20, playersOnline: 5 },
      { id: '2', name: 'Test Server 2', status: 'stopped', tps: 0, playersOnline: 0 }
    ];
    vi.mocked(api).mockResolvedValue(mockServers);

    render(
      <MockedRouter>
        <ServerList />
      </MockedRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Test Server 1')).toBeInTheDocument();
      expect(screen.getByText('Test Server 2')).toBeInTheDocument();
    });
  });

  it('should handle server actions correctly', async () => {
    const { api } = await import('../lib/api');
    const mockServers = [
      { id: '1', name: 'Test Server', status: 'running', tps: 20, playersOnline: 5 }
    ];
    vi.mocked(api).mockResolvedValue(mockServers);

    render(
      <MockedRouter>
        <ServerList />
      </MockedRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Test Server')).toBeInTheDocument();
    });

    // Test start button
    const startButton = screen.getByText('Start');
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(api).toHaveBeenCalledWith('/servers/1/start', { method: 'POST' });
    });
  });
});

describe('CreateServerModal Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render form fields correctly', () => {
    const onClose = vi.fn();
    const onSuccess = vi.fn();

    render(
      <MockedRouter>
        <CreateServerModal isOpen={true} onClose={onClose} onSuccess={onSuccess} />
      </MockedRouter>
    );

    expect(screen.getByLabelText('Server Name')).toBeInTheDocument();
    expect(screen.getByLabelText('Minecraft Version')).toBeInTheDocument();
    expect(screen.getByLabelText('Loader')).toBeInTheDocument();
    expect(screen.getByLabelText('Max Players')).toBeInTheDocument();
  });

  it('should validate form inputs', async () => {
    const onClose = vi.fn();
    const onSuccess = vi.fn();

    render(
      <MockedRouter>
        <CreateServerModal isOpen={true} onClose={onClose} onSuccess={onSuccess} />
      </MockedRouter>
    );

    const submitButton = screen.getByText('Create Server');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText('Server name is required')).toBeInTheDocument();
    });
  });

  it('should submit form with valid data', async () => {
    const { api } = await import('../lib/api');
    const onClose = vi.fn();
    const onSuccess = vi.fn();
    const mockResponse = { id: '1', name: 'New Server' };
    vi.mocked(api).mockResolvedValue(mockResponse);

    render(
      <MockedRouter>
        <CreateServerModal isOpen={true} onClose={onClose} onSuccess={onSuccess} />
      </MockedRouter>
    );

    // Fill form
    fireEvent.change(screen.getByLabelText('Server Name'), {
      target: { value: 'New Server' }
    });
    fireEvent.change(screen.getByLabelText('Minecraft Version'), {
      target: { value: '1.20.1' }
    });
    fireEvent.change(screen.getByLabelText('Loader'), {
      target: { value: 'vanilla' }
    });
    fireEvent.change(screen.getByLabelText('Max Players'), {
      target: { value: '20' }
    });

    const submitButton = screen.getByText('Create Server');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(api).toHaveBeenCalledWith('/servers', {
        method: 'POST',
        body: expect.objectContaining({
          name: 'New Server',
          minecraft_version: '1.20.1',
          loader: 'vanilla',
          max_players: 20
        })
      });
      expect(onSuccess).toHaveBeenCalled();
    });
  });
});
