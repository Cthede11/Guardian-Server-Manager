import { useState, useCallback } from 'react';

export interface Toast {
  id: string;
  title?: string;
  description?: string;
  variant?: 'default' | 'destructive' | 'success' | 'warning' | 'info';
  action?: React.ReactNode;
}

export interface ToastAction {
  toast: (toast: Omit<Toast, 'id'>) => void;
  dismiss: (toastId: string) => void;
}

let toastCount = 0;

export const useToast = (): ToastAction => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = useCallback((toastData: Omit<Toast, 'id'>) => {
    const id = (++toastCount).toString();
    const newToast: Toast = { ...toastData, id };
    
    setToasts(prev => [...prev, newToast]);
    
    // Auto-dismiss after 5 seconds
    setTimeout(() => {
      dismiss(id);
    }, 5000);
    
    return id;
  }, []);

  const dismiss = useCallback((toastId: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== toastId));
  }, []);

  return { toast, dismiss };
};
