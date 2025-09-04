import { toast } from '@/components/ui/use-toast';

export type NotificationType = 'success' | 'destructive' | 'warning' | 'info';

export interface NotificationOptions {
  title?: string;
  description?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

class NotificationService {
  private showToast(
    type: NotificationType,
    message: string,
    options: NotificationOptions = {}
  ) {
    const { title, description, duration } = options;

    toast({
      variant: type,
      title: title || this.getDefaultTitle(type),
      description: description || message,
      duration: duration || this.getDefaultDuration(type),
      // Note: Action functionality temporarily disabled due to type complexity
      // action: action ? {
      //   onClick: action.onClick,
      //   children: action.label,
      // } : undefined,
    });
  }

  private getDefaultTitle(type: NotificationType): string {
    switch (type) {
      case 'success':
        return 'Success';
      case 'destructive':
        return 'Error';
      case 'warning':
        return 'Warning';
      case 'info':
        return 'Information';
    }
  }

  private getDefaultDuration(type: NotificationType): number {
    switch (type) {
      case 'success':
        return 3000;
      case 'destructive':
        return 5000;
      case 'warning':
        return 4000;
      case 'info':
        return 3000;
    }
  }

  success(message: string, options?: NotificationOptions) {
    this.showToast('success', message, options);
  }

  error(message: string, options?: NotificationOptions) {
    this.showToast('destructive', message, options);
  }

  warning(message: string, options?: NotificationOptions) {
    this.showToast('warning', message, options);
  }

  info(message: string, options?: NotificationOptions) {
    this.showToast('info', message, options);
  }

  // Server-specific notifications
  serverStarted(serverName: string) {
    this.success(`Server "${serverName}" started successfully`);
  }

  serverStopped(serverName: string) {
    this.info(`Server "${serverName}" stopped`);
  }

  serverRestarted(serverName: string) {
    this.success(`Server "${serverName}" restarted successfully`);
  }

  serverPromoted(serverName: string) {
    this.success(`Server "${serverName}" promoted to production`);
  }

  serverActionFailed(serverName: string, action: string, error?: string) {
    this.error(
      `Failed to ${action} server "${serverName}"`,
      { description: error }
    );
  }

  // Player-specific notifications
  playerKicked(playerName: string) {
    this.success(`Player "${playerName}" kicked successfully`);
  }

  playerBanned(playerName: string) {
    this.warning(`Player "${playerName}" banned`);
  }

  playerTeleported(playerName: string) {
    this.info(`Player "${playerName}" teleported`);
  }

  playerThrottled(playerName: string) {
    this.warning(`Player "${playerName}" throttled`);
  }

  // Backup notifications
  backupCreated(serverName: string, backupName: string) {
    this.success(
      `Backup "${backupName}" created for server "${serverName}"`
    );
  }

  backupRestored(serverName: string, backupName: string) {
    this.success(
      `Backup "${backupName}" restored for server "${serverName}"`
    );
  }

  backupFailed(serverName: string, error?: string) {
    this.error(
      `Backup failed for server "${serverName}"`,
      { description: error }
    );
  }

  // Settings notifications
  settingsSaved(serverName: string) {
    this.success(`Settings saved for server "${serverName}"`);
  }

  settingsValidationFailed(errors: string[]) {
    this.error(
      'Settings validation failed',
      { description: errors.join(', ') }
    );
  }

  // Connection notifications
  connectionLost() {
    this.warning(
      'Connection lost',
      { description: 'Attempting to reconnect...' }
    );
  }

  connectionRestored() {
    this.success('Connection restored');
  }

  connectionFailed(error?: string) {
    this.error(
      'Failed to connect',
      { description: error }
    );
  }

  // Generic API notifications
  apiError(endpoint: string, error?: string) {
    this.error(
      `API request failed: ${endpoint}`,
      { description: error }
    );
  }

  apiSuccess(action: string) {
    this.success(`${action} completed successfully`);
  }

  // Form validation notifications
  formValidationFailed(field: string) {
    this.warning(`Please check the ${field} field`);
  }

  // File operation notifications
  fileUploaded(fileName: string) {
    this.success(`File "${fileName}" uploaded successfully`);
  }

  fileUploadFailed(fileName: string, error?: string) {
    this.error(
      `Failed to upload "${fileName}"`,
      { description: error }
    );
  }

  fileDeleted(fileName: string) {
    this.info(`File "${fileName}" deleted`);
  }
}

export const notifications = new NotificationService();
export default notifications;
