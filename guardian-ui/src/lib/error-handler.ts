export interface ErrorLog {
  id: string;
  timestamp: string;
  level: 'error' | 'warning' | 'info' | 'debug';
  message: string;
  stack?: string;
  context: string;
  userId?: string;
  serverId?: string;
  userAgent: string;
  url: string;
  resolved: boolean;
  resolvedAt?: string;
}

export interface ErrorReport {
  id: string;
  error: Error;
  context: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  userAction?: string;
  serverId?: string;
  timestamp: string;
}

export class ErrorHandler {
  private errorLogs: ErrorLog[] = [];
  private maxLogs = 1000;
  private isInitialized = false;

  async initialize(): Promise<void> {
    if (this.isInitialized) return;

    // Set up global error handlers
    window.addEventListener('error', (event) => {
      this.handleError(event.error, 'Global Error Handler', {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno
      });
    });

    window.addEventListener('unhandledrejection', (event) => {
      this.handleError(new Error(event.reason), 'Unhandled Promise Rejection');
    });

    this.isInitialized = true;
  }

  handleError(
    error: Error | string,
    context: string,
    additionalInfo?: any,
    severity: 'low' | 'medium' | 'high' | 'critical' = 'medium'
  ): void {
    const errorObj = typeof error === 'string' ? new Error(error) : error;
    
    const errorLog: ErrorLog = {
      id: this.generateId(),
      timestamp: new Date().toISOString(),
      level: this.getErrorLevel(severity),
      message: errorObj.message,
      stack: errorObj.stack,
      context,
      userAgent: navigator.userAgent,
      url: window.location.href,
      resolved: false
    };

    // Add additional info if provided
    if (additionalInfo) {
      errorLog.message += ` | Additional Info: ${JSON.stringify(additionalInfo)}`;
    }

    // Store error log
    this.errorLogs.push(errorLog);
    
    // Keep only the last maxLogs entries
    if (this.errorLogs.length > this.maxLogs) {
      this.errorLogs = this.errorLogs.slice(-this.maxLogs);
    }

    // Log to console
    console.error(`[${context}] ${errorObj.message}`, errorObj);

    // Save to file (if in Tauri environment)
    this.saveErrorToFile(errorLog);

    // Show user notification for critical errors
    if (severity === 'critical') {
      this.showUserNotification(errorObj.message, 'error');
    }
  }

  handleApiError(
    error: any,
    endpoint: string,
    method: string = 'GET',
    serverId?: string
  ): void {
    const context = `API Error - ${method} ${endpoint}`;
    const severity = this.getApiErrorSeverity(error);
    
    this.handleError(error, context, {
      endpoint,
      method,
      serverId,
      status: error.status,
      statusText: error.statusText
    }, severity);
  }

  handleServerError(
    error: any,
    serverId: string,
    operation: string
  ): void {
    const context = `Server Error - ${operation}`;
    const severity = this.getServerErrorSeverity(error);
    
    this.handleError(error, context, {
      serverId,
      operation
    }, severity);
  }

  handleFileSystemError(
    error: any,
    operation: string,
    filePath: string
  ): void {
    const context = `File System Error - ${operation}`;
    const severity = this.getFileSystemErrorSeverity(error);
    
    this.handleError(error, context, {
      operation,
      filePath
    }, severity);
  }

  getErrorLogs(limit?: number): ErrorLog[] {
    const logs = this.errorLogs.sort((a, b) => 
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
    
    return limit ? logs.slice(0, limit) : logs;
  }

  getErrorLogsByServer(serverId: string): ErrorLog[] {
    return this.errorLogs.filter(log => log.serverId === serverId);
  }

  getUnresolvedErrors(): ErrorLog[] {
    return this.errorLogs.filter(log => !log.resolved);
  }

  resolveError(errorId: string): void {
    const error = this.errorLogs.find(log => log.id === errorId);
    if (error) {
      error.resolved = true;
      error.resolvedAt = new Date().toISOString();
    }
  }

  clearResolvedErrors(): void {
    this.errorLogs = this.errorLogs.filter(log => !log.resolved);
  }

  exportErrorLogs(): string {
    return JSON.stringify(this.errorLogs, null, 2);
  }

  private getErrorLevel(severity: string): 'error' | 'warning' | 'info' | 'debug' {
    switch (severity) {
      case 'critical':
      case 'high':
        return 'error';
      case 'medium':
        return 'warning';
      case 'low':
        return 'info';
      default:
        return 'error';
    }
  }

  private getApiErrorSeverity(error: any): 'low' | 'medium' | 'high' | 'critical' {
    if (error.status >= 500) return 'high';
    if (error.status >= 400) return 'medium';
    return 'low';
  }

  private getServerErrorSeverity(error: any): 'low' | 'medium' | 'high' | 'critical' {
    if (error.message?.includes('crash') || error.message?.includes('fatal')) {
      return 'critical';
    }
    if (error.message?.includes('timeout') || error.message?.includes('connection')) {
      return 'high';
    }
    return 'medium';
  }

  private getFileSystemErrorSeverity(error: any): 'low' | 'medium' | 'high' | 'critical' {
    if (error.message?.includes('permission') || error.message?.includes('access')) {
      return 'high';
    }
    if (error.message?.includes('not found') || error.message?.includes('missing')) {
      return 'medium';
    }
    return 'low';
  }

  private async saveErrorToFile(errorLog: ErrorLog): Promise<void> {
    try {
      // TODO: Implement file saving using Tauri API
      console.log('Saving error to file:', errorLog);
    } catch (error) {
      console.error('Failed to save error to file:', error);
    }
  }

  private showUserNotification(message: string, type: 'error' | 'warning' | 'info' | 'success'): void {
    // TODO: Implement user notification system
    console.log(`[${type.toUpperCase()}] ${message}`);
  }

  private generateId(): string {
    return Math.random().toString(36).substr(2, 9);
  }
}

export const errorHandler = new ErrorHandler();

// Export individual functions for convenience
export const handleApiError = errorHandler.handleApiError.bind(errorHandler);
export const handleServerError = errorHandler.handleServerError.bind(errorHandler);
export const handleFileSystemError = errorHandler.handleFileSystemError.bind(errorHandler);

// Initialize error handler
errorHandler.initialize();