import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useThemeStore } from '@/store/theme';
import { 
  Palette, 
  Sun, 
  Moon, 
  Monitor, 
  Settings, 
  Save,
  RefreshCw,
  Download,
  Upload,
  Brush,
  Layers,
  Zap,
  Shield,
  Heart,
  Droplets,
  Flame,
  Leaf,
  Flower,
  Waves,
  Sun as SunIcon
} from 'lucide-react';

export const Theme: React.FC = () => {
  const theme = useThemeStore();
  const [isLoading, setIsLoading] = useState(false);

  const handleSave = async () => {
    setIsLoading(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      console.log('Theme settings saved successfully');
    } catch (error) {
      console.error('Failed to save theme settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReset = () => {
    theme.resetTheme();
  };

  const handleExport = () => {
    const dataStr = theme.exportTheme();
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    
    const exportFileDefaultName = 'guardian-theme-settings.json';
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (e) => {
          const success = theme.importTheme(e.target?.result as string);
          if (!success) {
            console.error('Failed to import theme settings');
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  };

  const getThemeIcon = (mode: string) => {
    switch (mode) {
      case 'light': return <Sun className="h-4 w-4" />;
      case 'dark': return <Moon className="h-4 w-4" />;
      case 'system': return <Monitor className="h-4 w-4" />;
      default: return <Sun className="h-4 w-4" />;
    }
  };

  const getColorSchemeIcon = (scheme: string) => {
    switch (scheme) {
      case 'blue': return <Droplets className="h-4 w-4" />;
      case 'green': return <Leaf className="h-4 w-4" />;
      case 'purple': return <Flower className="h-4 w-4" />;
      case 'red': return <Flame className="h-4 w-4" />;
      case 'orange': return <SunIcon className="h-4 w-4" />;
      default: return <Palette className="h-4 w-4" />;
    }
  };

  return (
    <div className="h-full flex flex-col space-y-8 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-bold gradient-text">Theme Settings</h1>
          <p className="text-muted-foreground mt-3 text-lg">
            Customize the appearance and behavior of your Guardian interface
          </p>
        </div>
        <div className="flex gap-3">
          <Button variant="outline" onClick={handleReset} className="shadow-sm hover:shadow-md transition-all duration-200">
            <RefreshCw className="h-4 w-4 mr-2" />
            Reset
          </Button>
          <Button variant="outline" onClick={handleExport} className="shadow-sm hover:shadow-md transition-all duration-200">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          <Button variant="outline" onClick={handleImport} className="shadow-sm hover:shadow-md transition-all duration-200">
            <Upload className="h-4 w-4 mr-2" />
            Import
          </Button>
          <Button onClick={handleSave} disabled={isLoading} className="bg-primary hover:bg-primary/90 shadow-md hover:shadow-lg transition-all duration-200">
            <Save className="h-4 w-4 mr-2" />
            {isLoading ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>

      {/* Theme Selection */}
      <Card className="modern-card">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Palette className="h-5 w-5" />
            Theme Selection
          </CardTitle>
          <CardDescription>
            Choose your preferred theme and color scheme
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <Label htmlFor="theme-mode">Theme Mode</Label>
              <Select value={theme.mode} onValueChange={(value) => theme.updateTheme({ mode: value as any })}>
                <SelectTrigger>
                  <SelectValue placeholder="Select theme mode" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="light">
                    <div className="flex items-center gap-2">
                      {getThemeIcon('light')}
                      Light
                    </div>
                  </SelectItem>
                  <SelectItem value="dark">
                    <div className="flex items-center gap-2">
                      {getThemeIcon('dark')}
                      Dark
                    </div>
                  </SelectItem>
                  <SelectItem value="system">
                    <div className="flex items-center gap-2">
                      {getThemeIcon('system')}
                      System
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="color-scheme">Color Scheme</Label>
              <Select value={theme.colorScheme} onValueChange={(value) => theme.updateTheme({ colorScheme: value as any })}>
                <SelectTrigger>
                  <SelectValue placeholder="Select color scheme" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="default">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('default')}
                      Default
                    </div>
                  </SelectItem>
                  <SelectItem value="blue">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('blue')}
                      Blue
                    </div>
                  </SelectItem>
                  <SelectItem value="green">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('green')}
                      Green
                    </div>
                  </SelectItem>
                  <SelectItem value="purple">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('purple')}
                      Purple
                    </div>
                  </SelectItem>
                  <SelectItem value="red">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('red')}
                      Red
                    </div>
                  </SelectItem>
                  <SelectItem value="orange">
                    <div className="flex items-center gap-2">
                      {getColorSchemeIcon('orange')}
                      Orange
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

        </CardContent>
      </Card>

      {/* UI Preferences */}
      <Card className="modern-card">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            UI Preferences
          </CardTitle>
          <CardDescription>
            Configure interface behavior and display options
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="compact-mode">Compact Mode</Label>
                  <p className="text-sm text-muted-foreground">Use smaller spacing and components</p>
                </div>
                <Switch
                  id="compact-mode"
                  checked={theme.compactMode}
                  onCheckedChange={(checked) => theme.updateTheme({ compactMode: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="show-animations">Show Animations</Label>
                  <p className="text-sm text-muted-foreground">Enable smooth transitions and animations</p>
                </div>
                <Switch
                  id="show-animations"
                  checked={theme.showAnimations}
                  onCheckedChange={(checked) => theme.updateTheme({ showAnimations: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="show-tooltips">Show Tooltips</Label>
                  <p className="text-sm text-muted-foreground">Display helpful tooltips on hover</p>
                </div>
                <Switch
                  id="show-tooltips"
                  checked={theme.showTooltips}
                  onCheckedChange={(checked) => theme.updateTheme({ showTooltips: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="show-notifications">Show Notifications</Label>
                  <p className="text-sm text-muted-foreground">Display system notifications</p>
                </div>
                <Switch
                  id="show-notifications"
                  checked={theme.showNotifications}
                  onCheckedChange={(checked) => theme.updateTheme({ showNotifications: checked })}
                />
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="status-indicators">Status Indicators</Label>
                  <p className="text-sm text-muted-foreground">Show status indicators and badges</p>
                </div>
                <Switch
                  id="status-indicators"
                  checked={theme.statusIndicators}
                  onCheckedChange={(checked) => theme.updateTheme({ statusIndicators: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="progress-bars">Progress Bars</Label>
                  <p className="text-sm text-muted-foreground">Display progress indicators</p>
                </div>
                <Switch
                  id="progress-bars"
                  checked={theme.progressBars}
                  onCheckedChange={(checked) => theme.updateTheme({ progressBars: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="breadcrumbs">Breadcrumbs</Label>
                  <p className="text-sm text-muted-foreground">Show navigation breadcrumbs</p>
                </div>
                <Switch
                  id="breadcrumbs"
                  checked={theme.breadcrumbs}
                  onCheckedChange={(checked) => theme.updateTheme({ breadcrumbs: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="search-suggestions">Search Suggestions</Label>
                  <p className="text-sm text-muted-foreground">Show search suggestions and autocomplete</p>
                </div>
                <Switch
                  id="search-suggestions"
                  checked={theme.searchSuggestions}
                  onCheckedChange={(checked) => theme.updateTheme({ searchSuggestions: checked })}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Accessibility */}
      <Card className="modern-card">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Accessibility
          </CardTitle>
          <CardDescription>
            Configure accessibility features and options
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="high-contrast">High Contrast</Label>
                  <p className="text-sm text-muted-foreground">Use high contrast colors</p>
                </div>
                <Switch
                  id="high-contrast"
                  checked={theme.highContrast}
                  onCheckedChange={(checked) => theme.updateTheme({ highContrast: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="reduced-motion">Reduced Motion</Label>
                  <p className="text-sm text-muted-foreground">Reduce animations and transitions</p>
                </div>
                <Switch
                  id="reduced-motion"
                  checked={theme.reducedMotion}
                  onCheckedChange={(checked) => theme.updateTheme({ reducedMotion: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="large-text">Large Text</Label>
                  <p className="text-sm text-muted-foreground">Use larger text sizes</p>
                </div>
                <Switch
                  id="large-text"
                  checked={theme.largeText}
                  onCheckedChange={(checked) => theme.updateTheme({ largeText: checked })}
                />
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="screen-reader">Screen Reader</Label>
                  <p className="text-sm text-muted-foreground">Optimize for screen readers</p>
                </div>
                <Switch
                  id="screen-reader"
                  checked={theme.screenReader}
                  onCheckedChange={(checked) => theme.updateTheme({ screenReader: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="keyboard-navigation">Keyboard Navigation</Label>
                  <p className="text-sm text-muted-foreground">Enable keyboard navigation</p>
                </div>
                <Switch
                  id="keyboard-navigation"
                  checked={theme.keyboardNavigation}
                  onCheckedChange={(checked) => theme.updateTheme({ keyboardNavigation: checked })}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="focus-indicators">Focus Indicators</Label>
                  <p className="text-sm text-muted-foreground">Show focus indicators</p>
                </div>
                <Switch
                  id="focus-indicators"
                  checked={theme.focusIndicators}
                  onCheckedChange={(checked) => theme.updateTheme({ focusIndicators: checked })}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Advanced */}
      <Card className="modern-card">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5" />
            Advanced
          </CardTitle>
          <CardDescription>
            Advanced customization and development options
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="developer-tools">Developer Tools</Label>
                <p className="text-sm text-muted-foreground">Enable development tools</p>
              </div>
              <Switch
                id="developer-tools"
                checked={theme.developerTools}
                onCheckedChange={(checked) => theme.updateTheme({ developerTools: checked })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="debug-mode">Debug Mode</Label>
                <p className="text-sm text-muted-foreground">Enable debug logging</p>
              </div>
              <Switch
                id="debug-mode"
                checked={theme.debugMode}
                onCheckedChange={(checked) => theme.updateTheme({ debugMode: checked })}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="performance-mode">Performance Mode</Label>
                <p className="text-sm text-muted-foreground">Optimize for performance</p>
              </div>
              <Switch
                id="performance-mode"
                checked={theme.performanceMode}
                onCheckedChange={(checked) => theme.updateTheme({ performanceMode: checked })}
              />
            </div>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="custom-css">Custom CSS</Label>
              <Textarea
                id="custom-css"
                placeholder="/* Custom CSS styles */"
                value={theme.customCSS}
                onChange={(e) => theme.updateTheme({ customCSS: e.target.value })}
                className="min-h-[120px] font-mono text-sm"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="custom-javascript">Custom JavaScript</Label>
              <Textarea
                id="custom-javascript"
                placeholder="// Custom JavaScript code"
                value={theme.customJavaScript}
                onChange={(e) => theme.updateTheme({ customJavaScript: e.target.value })}
                className="min-h-[120px] font-mono text-sm"
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
