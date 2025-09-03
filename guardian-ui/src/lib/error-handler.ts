import { notifications } from './notifications';

export interface ApiError {
  message: string;
  code?: string;
  status?: number;
  details?: any;
}

export class GuardianError extends Error {
  public code?: string;
  public status?: number;
  public details?: any;

  constructor(message: string, code?: string, status?: number, details?: any) {
    super(message);
    this.name = 'GuardianError';
    this.code = code;
    this.status = status;
    this.details = details;
  }
}

export const isApiError = (error: any): error is ApiError => {
  return error && typeof error === 'object' && 'message' in error;
};

export const isGuardianError = (error: any): error is GuardianError => {
  return error instanceof GuardianError;
};

export const createApiError = (error: any): GuardianError => {
  if (isGuardianError(error)) {
    return error;
  }

  if (isApiError(error)) {
    return new GuardianError(
      error.message,
      error.code,
      error.status,
      error.details
    );
  }

  if (error instanceof Error) {
    return new GuardianError(error.message);
  }

  return new GuardianError('An unknown error occurred');
};

export const handleApiError = (error: any, context?: string): GuardianError => {
  const guardianError = createApiError(error);
  
  // Log error for debugging
  console.error(`API Error${context ? ` in ${context}` : ''}:`, guardianError);

  // Show user-friendly notification
  const message = context 
    ? `${context}: ${guardianError.message}`
    : guardianError.message;

  notifications.error(message, {
    description: guardianError.details ? 
      JSON.stringify(guardianError.details, null, 2) : 
      undefined
  });

  return guardianError;
};

export const handleNetworkError = (error: any): GuardianError => {
  const guardianError = createApiError(error);
  
  // Check for common network error patterns
  if (error.message?.includes('fetch')) {
    guardianError.message = 'Network connection failed. Please check your internet connection.';
  } else if (error.message?.includes('timeout')) {
    guardianError.message = 'Request timed out. Please try again.';
  } else if (error.message?.includes('CORS')) {
    guardianError.message = 'Cross-origin request blocked. Please contact support.';
  }

  notifications.error(guardianError.message);
  return guardianError;
};

export const handleValidationError = (error: any): GuardianError => {
  const guardianError = createApiError(error);
  
  // Extract validation errors from response
  if (error.details && Array.isArray(error.details)) {
    const validationErrors = error.details.map((detail: any) => 
      `${detail.field}: ${detail.message}`
    ).join(', ');
    
    guardianError.message = `Validation failed: ${validationErrors}`;
  }

  notifications.warning(guardianError.message);
  return guardianError;
};

export const handleServerError = (error: any, serverName?: string): GuardianError => {
  const guardianError = createApiError(error);
  
  const context = serverName ? `Server "${serverName}"` : 'Server';
  
  // Check for specific server error codes
  switch (guardianError.code) {
    case 'SERVER_NOT_FOUND':
      guardianError.message = `${context} not found`;
      break;
    case 'SERVER_ALREADY_RUNNING':
      guardianError.message = `${context} is already running`;
      break;
    case 'SERVER_NOT_RUNNING':
      guardianError.message = `${context} is not running`;
      break;
    case 'INSUFFICIENT_RESOURCES':
      guardianError.message = `Insufficient resources for ${context}`;
      break;
    case 'BACKUP_FAILED':
      guardianError.message = `Backup failed for ${context}`;
      break;
    case 'RESTORE_FAILED':
      guardianError.message = `Restore failed for ${context}`;
      break;
  }

  notifications.error(guardianError.message, {
    description: guardianError.details ? 
      JSON.stringify(guardianError.details, null, 2) : 
      undefined
  });

  return guardianError;
};

export const handleConnectionError = (error: any): GuardianError => {
  const guardianError = createApiError(error);
  
  // Check for connection-specific errors
  if (error.message?.includes('WebSocket')) {
    guardianError.message = 'Real-time connection lost. Falling back to polling.';
    notifications.warning(guardianError.message);
  } else if (error.message?.includes('SSE')) {
    guardianError.message = 'Server-sent events connection failed.';
    notifications.error(guardianError.message);
  } else {
    guardianError.message = 'Connection error occurred.';
    notifications.error(guardianError.message);
  }

  return guardianError;
};

// Utility function to safely execute async operations with error handling
export const safeAsync = async <T>(
  operation: () => Promise<T>,
  errorHandler?: (error: any) => void
): Promise<T | null> => {
  try {
    return await operation();
  } catch (error) {
    if (errorHandler) {
      errorHandler(error);
    } else {
      handleApiError(error);
    }
    return null;
  }
};

// Utility function to retry operations with exponential backoff
export const retryOperation = async <T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> => {
  let lastError: any;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;
      
      if (attempt === maxRetries) {
        break;
      }

      const delay = baseDelay * Math.pow(2, attempt);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError;
};

// Error boundary helper for React components
export const withErrorHandling = <P extends object>(
  Component: React.ComponentType<P>,
  errorContext?: string
) => {
  return (props: P) => {
    try {
      return <Component {...props} />;
    } catch (error) {
      const guardianError = handleApiError(error, errorContext);
      throw guardianError;
    }
  };
};
