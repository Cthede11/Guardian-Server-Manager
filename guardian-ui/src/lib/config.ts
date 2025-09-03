export const config = {
  apiUrl: import.meta.env.VITE_API_URL || '/api/v1',
  useSSE: import.meta.env.VITE_USE_SSE === 'true' || false,
  enableMocks: import.meta.env.VITE_ENABLE_MOCKS === 'true' || true,
};
