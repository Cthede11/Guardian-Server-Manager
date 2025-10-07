import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface LogEntry {
  id: string;
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  data?: any[];
}

interface ConsoleState {
  logs: LogEntry[];
  addLog: (level: LogEntry['level'], ...args: any[]) => void;
  clearLogs: () => void;
  getLogs: () => LogEntry[];
}

export const useConsoleStore = create<ConsoleState>()(
  persist(
    (set, get) => ({
      logs: [],
      
      addLog: (level: LogEntry['level'], ...args: any[]) => {
        const id = `log-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        const timestamp = new Date().toLocaleTimeString();
        const message = args.map(arg => 
          typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
        ).join(' ');
        
        set(state => ({
          logs: [...state.logs, { id, timestamp, level, message, data: args }]
        }));
      },
      
      clearLogs: () => {
        set({ logs: [] });
      },
      
      getLogs: () => {
        return get().logs;
      }
    }),
    {
      name: 'console-logs',
      // Only persist the last 1000 logs to prevent storage bloat
      partialize: (state) => ({
        logs: state.logs.slice(-1000)
      })
    }
  )
);
