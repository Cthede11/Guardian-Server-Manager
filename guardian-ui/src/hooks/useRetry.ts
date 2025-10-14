import { useState, useCallback, useRef } from 'react';

export interface RetryOptions {
  maxRetries?: number;
  baseDelay?: number;
  maxDelay?: number;
  backoffMultiplier?: number;
  onRetry?: (attempt: number, error: Error) => void;
  onSuccess?: () => void;
  onFailure?: (error: Error) => void;
}

export interface RetryState {
  isLoading: boolean;
  error: Error | null;
  retryCount: number;
  isRetrying: boolean;
}

export function useRetry<T extends any[], R>(
  asyncFunction: (...args: T) => Promise<R>,
  options: RetryOptions = {}
) {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    maxDelay = 10000,
    backoffMultiplier = 2,
    onRetry,
    onSuccess,
    onFailure
  } = options;

  const [state, setState] = useState<RetryState>({
    isLoading: false,
    error: null,
    retryCount: 0,
    isRetrying: false
  });

  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const calculateDelay = useCallback((attempt: number): number => {
    const delay = baseDelay * Math.pow(backoffMultiplier, attempt);
    return Math.min(delay, maxDelay);
  }, [baseDelay, backoffMultiplier, maxDelay]);

  const execute = useCallback(async (...args: T): Promise<R | null> => {
    setState(prev => ({
      ...prev,
      isLoading: true,
      error: null,
      isRetrying: false
    }));

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        const result = await asyncFunction(...args);
        
        setState(prev => ({
          ...prev,
          isLoading: false,
          error: null,
          retryCount: 0,
          isRetrying: false
        }));

        onSuccess?.();
        return result;

      } catch (error) {
        lastError = error instanceof Error ? error : new Error('Unknown error');
        
        if (attempt === maxRetries) {
          // Final attempt failed
          setState(prev => ({
            ...prev,
            isLoading: false,
            error: lastError,
            isRetrying: false
          }));
          
          onFailure?.(lastError);
          return null;
        }

        // Schedule retry
        setState(prev => ({
          ...prev,
          retryCount: attempt + 1,
          isRetrying: true
        }));

        onRetry?.(attempt + 1, lastError);

        const delay = calculateDelay(attempt);
        await new Promise(resolve => {
          timeoutRef.current = setTimeout(resolve, delay);
        });
      }
    }

    return null;
  }, [asyncFunction, maxRetries, calculateDelay, onRetry, onSuccess, onFailure]);

  const reset = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    
    setState({
      isLoading: false,
      error: null,
      retryCount: 0,
      isRetrying: false
    });
  }, []);

  return {
    ...state,
    execute,
    reset
  };
}

export default useRetry;
