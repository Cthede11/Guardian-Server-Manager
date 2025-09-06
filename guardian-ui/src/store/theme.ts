import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ColorScheme = 'default' | 'blue' | 'green' | 'purple' | 'red' | 'orange';

export interface ThemeSettings {
  mode: ThemeMode;
  colorScheme: ColorScheme;
  compactMode: boolean;
  showAnimations: boolean;
  showTooltips: boolean;
  showNotifications: boolean;
  statusIndicators: boolean;
  progressBars: boolean;
  breadcrumbs: boolean;
  searchSuggestions: boolean;
  highContrast: boolean;
  reducedMotion: boolean;
  largeText: boolean;
  screenReader: boolean;
  keyboardNavigation: boolean;
  focusIndicators: boolean;
  developerTools: boolean;
  debugMode: boolean;
  performanceMode: boolean;
  customCSS: string;
  customJavaScript: string;
}

const defaultTheme: ThemeSettings = {
  mode: 'system',
  colorScheme: 'default',
  compactMode: false,
  showAnimations: true,
  showTooltips: true,
  showNotifications: true,
  statusIndicators: true,
  progressBars: true,
  breadcrumbs: true,
  searchSuggestions: true,
  highContrast: false,
  reducedMotion: false,
  largeText: false,
  screenReader: false,
  keyboardNavigation: true,
  focusIndicators: true,
  developerTools: false,
  debugMode: false,
  performanceMode: false,
  customCSS: '/* Custom CSS styles */',
  customJavaScript: '// Custom JavaScript code',
};

export const useThemeStore = create<ThemeSettings & {
  updateTheme: (updates: Partial<ThemeSettings>) => void;
  resetTheme: () => void;
  exportTheme: () => string;
  importTheme: (themeData: string) => boolean;
}>()(
  persist(
    (set, get) => ({
      ...defaultTheme,
      
      updateTheme: (updates) => {
        set(updates);
        // Apply theme changes to the DOM
        applyThemeToDOM({ ...get(), ...updates });
      },
      
      resetTheme: () => {
        set(defaultTheme);
        applyThemeToDOM(defaultTheme);
      },
      
      exportTheme: () => {
        const theme = get();
        return JSON.stringify(theme, null, 2);
      },
      
      importTheme: (themeData) => {
        try {
          const theme = JSON.parse(themeData);
          set(theme);
          applyThemeToDOM(theme);
          return true;
        } catch (error) {
          console.error('Failed to import theme:', error);
          return false;
        }
      },
    }),
    {
      name: 'guardian-theme',
    }
  )
);

// Apply theme settings to the DOM
function applyThemeToDOM(theme: ThemeSettings) {
  const root = document.documentElement;
  
  // Apply theme mode
  if (theme.mode === 'system') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
  } else {
    root.setAttribute('data-theme', theme.mode);
  }
  
  // Apply color scheme
  root.setAttribute('data-color-scheme', theme.colorScheme);
  
  // Apply accessibility settings
  root.setAttribute('data-high-contrast', theme.highContrast.toString());
  root.setAttribute('data-reduced-motion', theme.reducedMotion.toString());
  root.setAttribute('data-large-text', theme.largeText.toString());
  
  // Apply UI preferences
  root.setAttribute('data-compact-mode', theme.compactMode.toString());
  root.setAttribute('data-show-animations', theme.showAnimations.toString());
  root.setAttribute('data-show-tooltips', theme.showTooltips.toString());
  root.setAttribute('data-show-notifications', theme.showNotifications.toString());
  root.setAttribute('data-status-indicators', theme.statusIndicators.toString());
  root.setAttribute('data-progress-bars', theme.progressBars.toString());
  root.setAttribute('data-breadcrumbs', theme.breadcrumbs.toString());
  root.setAttribute('data-search-suggestions', theme.searchSuggestions.toString());
  
  // Apply custom CSS
  let customStyleElement = document.getElementById('custom-theme-styles');
  if (!customStyleElement) {
    customStyleElement = document.createElement('style');
    customStyleElement.id = 'custom-theme-styles';
    document.head.appendChild(customStyleElement);
  }
  customStyleElement.textContent = theme.customCSS;
  
  // Apply custom JavaScript
  if (theme.customJavaScript && theme.customJavaScript !== '// Custom JavaScript code') {
    try {
      // Remove previous custom script
      const existingScript = document.getElementById('custom-theme-script');
      if (existingScript) {
        existingScript.remove();
      }
      
      // Add new custom script
      const script = document.createElement('script');
      script.id = 'custom-theme-script';
      script.textContent = theme.customJavaScript;
      document.head.appendChild(script);
    } catch (error) {
      console.error('Failed to apply custom JavaScript:', error);
    }
  }
}

// Listen for system theme changes
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const theme = useThemeStore.getState();
    if (theme.mode === 'system') {
      applyThemeToDOM(theme);
    }
  });
}
