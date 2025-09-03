import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  Palette, 
  Sun, 
  Moon, 
  Monitor, 
  Settings, 
  AlertTriangle,
  CheckCircle,
  Info,
  Save,
  RefreshCw,
  Eye,
  Download,
  Upload,
  FileText,
  Brush,
  Paintbrush,
  Layers,
  Zap,
  Globe,
  User,
  Shield,
  Database,
  Server,
  Network,
  Activity,
  Clock,
  Star,
  Heart,
  Sparkles,
  Rainbow,
  Droplets,
  Flame,
  Snowflake,
  Leaf,
  Flower,
  Mountain,
  Waves,
  Cloud,
  Sun as SunIcon,
  Moon as MoonIcon,
  Sunrise,
  Sunset
} from 'lucide-react';

interface ThemeSettings {
  // Theme Selection
  theme: 'light' | 'dark' | 'auto';
  colorScheme: 'default' | 'blue' | 'green' | 'purple' | 'orange' | 'red' | 'pink' | 'cyan' | 'custom';
  
  // Custom Colors
  customColors: {
    primary: string;
    secondary: string;
    accent: string;
    background: string;
    foreground: string;
    muted: string;
    border: string;
    input: string;
    ring: string;
  };
  
  // Layout Settings
  layout: {
    sidebarWidth: number;
    headerHeight: number;
    borderRadius: number;
    spacing: number;
    fontSize: number;
    lineHeight: number;
  };
  
  // UI Preferences
  preferences: {
    compactMode: boolean;
    showAnimations: boolean;
    showTooltips: boolean;
    showNotifications: boolean;
    showStatusIndicators: boolean;
    showProgressBars: boolean;
    showBreadcrumbs: boolean;
    showSearchSuggestions: boolean;
  };
  
  // Accessibility
  accessibility: {
    highContrast: boolean;
    reducedMotion: boolean;
    largeText: boolean;
    screenReader: boolean;
    keyboardNavigation: boolean;
    focusIndicators: boolean;
  };
  
  // Advanced
  advanced: {
    customCSS: string;
    customJS: string;
    enableDevTools: boolean;
    enableDebugMode: boolean;
    enablePerformanceMode: boolean;
  };
}

export const Theme: React.FC = () => {
  const [settings, setSettings] = useState<ThemeSettings>({
    // Theme Selection
    theme: 'dark',
    colorScheme: 'default',
    
    // Custom Colors
    customColors: {
      primary: '#3b82f6',
      secondary: '#64748b',
      accent: '#f59e0b',
      background: '#0f172a',
      foreground: '#f8fafc',
      muted: '#1e293b',
      border: '#334155',
      input: '#1e293b',
      ring: '#3b82f6'
    },
    
    // Layout Settings
    layout: {
      sidebarWidth: 280,
      headerHeight: 64,
      borderRadius: 8,
      spacing: 16,
      fontSize: 14,
      lineHeight: 1.5
    },
    
    // UI Preferences
    preferences: {
      compactMode: false,
      showAnimations: true,
      showTooltips: true,
      showNotifications: true,
      showStatusIndicators: true,
      showProgressBars: true,
      showBreadcrumbs: true,
      showSearchSuggestions: true
    },
    
    // Accessibility
    accessibility: {
      highContrast: false,
      reducedMotion: false,
      largeText: false,
      screenReader: false,
      keyboardNavigation: true,
      focusIndicators: true
    },
    
    // Advanced
    advanced: {
      customCSS: '',
      customJS: '',
      enableDevTools: false,
      enableDebugMode: false,
      enablePerformanceMode: false
    }
  });
  
  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [previewMode, setPreviewMode] = useState(false);

  const fetchSettings = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch theme settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSettingChange = (key: keyof ThemeSettings, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const handleNestedSettingChange = (parentKey: keyof ThemeSettings, childKey: string, value: any) => {
    setSettings(prev => ({
      ...prev,
      [parentKey]: {
        ...prev[parentKey],
        [childKey]: value
      }
    }));
    setHasChanges(true);
  };

  const getThemeIcon = (theme: string) => {
    switch (theme) {
      case 'light': return <Sun className="h-4 w-4" />;
      case 'dark': return <Moon className="h-4 w-4" />;
      case 'auto': return <Monitor className="h-4 w-4" />;
      default: return <Palette className="h-4 w-4" />;
    }
  };

  const getColorSchemeIcon = (scheme: string) => {
    switch (scheme) {
      case 'default': return <Palette className="h-4 w-4" />;
      case 'blue': return <Droplets className="h-4 w-4" />;
      case 'green': return <Leaf className="h-4 w-4" />;
      case 'purple': return <Flower className="h-4 w-4" />;
      case 'orange': return <SunIcon className="h-4 w-4" />;
      case 'red': return <Flame className="h-4 w-4" />;
      case 'pink': return <Heart className="h-4 w-4" />;
      case 'cyan': return <Waves className="h-4 w-4" />;
      case 'custom': return <Brush className="h-4 w-4" />;
      default: return <Palette className="h-4 w-4" />;
    }
  };

  const getColorSchemeColor = (scheme: string) => {
    switch (scheme) {
      case 'default': return 'bg-gray-500';
      case 'blue': return 'bg-blue-500';
      case 'green': return 'bg-green-500';
      case 'purple': return 'bg-purple-500';
      case 'orange': return 'bg-orange-500';
      case 'red': return 'bg-red-500';
      case 'pink': return 'bg-pink-500';
      case 'cyan': return 'bg-cyan-500';
      case 'custom': return 'bg-gradient-to-r from-purple-500 to-pink-500';
      default: return 'bg-gray-500';
    }
  };

  const getColorSchemeLabel = (scheme: string) => {
    switch (scheme) {
      case 'default': return 'Default';
      case 'blue': return 'Ocean';
      case 'green': return 'Forest';
      case 'purple': return 'Royal';
      case 'orange': return 'Sunset';
      case 'red': return 'Fire';
      case 'pink': return 'Rose';
      case 'cyan': return 'Aqua';
      case 'custom': return 'Custom';
      default: return 'Unknown';
    }
  };

  const handleSaveSettings = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to save theme settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleResetSettings = () => {
    // Reset to default settings
    setSettings({
      theme: 'dark',
      colorScheme: 'default',
      customColors: {
        primary: '#3b82f6',
        secondary: '#64748b',
        accent: '#f59e0b',
        background: '#0f172a',
        foreground: '#f8fafc',
        muted: '#1e293b',
        border: '#334155',
        input: '#1e293b',
        ring: '#3b82f6'
      },
      layout: {
        sidebarWidth: 280,
        headerHeight: 64,
        borderRadius: 8,
        spacing: 16,
        fontSize: 14,
        lineHeight: 1.5
      },
      preferences: {
        compactMode: false,
        showAnimations: true,
        showTooltips: true,
        showNotifications: true,
        showStatusIndicators: true,
        showProgressBars: true,
        showBreadcrumbs: true,
        showSearchSuggestions: true
      },
      accessibility: {
        highContrast: false,
        reducedMotion: false,
        largeText: false,
        screenReader: false,
        keyboardNavigation: true,
        focusIndicators: true
      },
      advanced: {
        customCSS: '',
        customJS: '',
        enableDevTools: false,
        enableDebugMode: false,
        enablePerformanceMode: false
      }
    });
    setHasChanges(true);
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Theme Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Palette className="h-5 w-5" />
            <span>Theme Selection</span>
          </CardTitle>
          <CardDescription>
            Choose your preferred theme and color scheme
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="theme">Theme Mode</Label>
                <Select value={settings.theme} onValueChange={(value) => handleSettingChange('theme', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="light">
                      <div className="flex items-center space-x-2">
                        <Sun className="h-4 w-4" />
                        <span>Light</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="dark">
                      <div className="flex items-center space-x-2">
                        <Moon className="h-4 w-4" />
                        <span>Dark</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="auto">
                      <div className="flex items-center space-x-2">
                        <Monitor className="h-4 w-4" />
                        <span>Auto</span>
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="colorScheme">Color Scheme</Label>
                <Select value={settings.colorScheme} onValueChange={(value) => handleSettingChange('colorScheme', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">
                      <div className="flex items-center space-x-2">
                        <Palette className="h-4 w-4" />
                        <span>Default</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="blue">
                      <div className="flex items-center space-x-2">
                        <Droplets className="h-4 w-4" />
                        <span>Ocean</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="green">
                      <div className="flex items-center space-x-2">
                        <Leaf className="h-4 w-4" />
                        <span>Forest</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="purple">
                      <div className="flex items-center space-x-2">
                        <Flower className="h-4 w-4" />
                        <span>Royal</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="orange">
                      <div className="flex items-center space-x-2">
                        <SunIcon className="h-4 w-4" />
                        <span>Sunset</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="red">
                      <div className="flex items-center space-x-2">
                        <Flame className="h-4 w-4" />
                        <span>Fire</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="pink">
                      <div className="flex items-center space-x-2">
                        <Heart className="h-4 w-4" />
                        <span>Rose</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="cyan">
                      <div className="flex items-center space-x-2">
                        <Waves className="h-4 w-4" />
                        <span>Aqua</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="custom">
                      <div className="flex items-center space-x-2">
                        <Brush className="h-4 w-4" />
                        <span>Custom</span>
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label>Preview</Label>
                <div className="p-4 border rounded-lg bg-muted">
                  <div className="flex items-center space-x-2 mb-2">
                    <div className={`w-3 h-3 rounded-full ${getColorSchemeColor(settings.colorScheme)}`} />
                    <span className="text-sm font-medium">{getColorSchemeLabel(settings.colorScheme)}</span>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {settings.theme === 'auto' ? 'Follows system preference' : `${settings.theme} theme`}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Custom Colors */}
      {settings.colorScheme === 'custom' && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Brush className="h-5 w-5" />
              <span>Custom Colors</span>
            </CardTitle>
            <CardDescription>
              Customize your color palette
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {Object.entries(settings.customColors).map(([key, value]) => (
                <div key={key} className="space-y-2">
                  <Label htmlFor={key} className="capitalize">{key}</Label>
                  <div className="flex items-center space-x-2">
                    <Input
                      id={key}
                      value={value}
                      onChange={(e) => handleNestedSettingChange('customColors', key, e.target.value)}
                      className="font-mono text-sm"
                    />
                    <div 
                      className="w-8 h-8 rounded border"
                      style={{ backgroundColor: value }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Layout Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Layers className="h-5 w-5" />
            <span>Layout Settings</span>
          </CardTitle>
          <CardDescription>
            Customize the layout and spacing
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <Label htmlFor="sidebarWidth">Sidebar Width (px)</Label>
              <Input
                id="sidebarWidth"
                type="number"
                value={settings.layout.sidebarWidth}
                onChange={(e) => handleNestedSettingChange('layout', 'sidebarWidth', parseInt(e.target.value))}
                min="200"
                max="400"
              />
            </div>
            
            <div>
              <Label htmlFor="headerHeight">Header Height (px)</Label>
              <Input
                id="headerHeight"
                type="number"
                value={settings.layout.headerHeight}
                onChange={(e) => handleNestedSettingChange('layout', 'headerHeight', parseInt(e.target.value))}
                min="48"
                max="96"
              />
            </div>
            
            <div>
              <Label htmlFor="borderRadius">Border Radius (px)</Label>
              <Input
                id="borderRadius"
                type="number"
                value={settings.layout.borderRadius}
                onChange={(e) => handleNestedSettingChange('layout', 'borderRadius', parseInt(e.target.value))}
                min="0"
                max="24"
              />
            </div>
            
            <div>
              <Label htmlFor="spacing">Spacing (px)</Label>
              <Input
                id="spacing"
                type="number"
                value={settings.layout.spacing}
                onChange={(e) => handleNestedSettingChange('layout', 'spacing', parseInt(e.target.value))}
                min="8"
                max="32"
              />
            </div>
            
            <div>
              <Label htmlFor="fontSize">Font Size (px)</Label>
              <Input
                id="fontSize"
                type="number"
                value={settings.layout.fontSize}
                onChange={(e) => handleNestedSettingChange('layout', 'fontSize', parseInt(e.target.value))}
                min="12"
                max="18"
              />
            </div>
            
            <div>
              <Label htmlFor="lineHeight">Line Height</Label>
              <Input
                id="lineHeight"
                type="number"
                step="0.1"
                value={settings.layout.lineHeight}
                onChange={(e) => handleNestedSettingChange('layout', 'lineHeight', parseFloat(e.target.value))}
                min="1.0"
                max="2.0"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* UI Preferences */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Settings className="h-5 w-5" />
            <span>UI Preferences</span>
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
                  <Label htmlFor="compactMode">Compact Mode</Label>
                  <p className="text-sm text-muted-foreground">Use smaller spacing and components</p>
                </div>
                <Switch
                  id="compactMode"
                  checked={settings.preferences.compactMode}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'compactMode', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showAnimations">Show Animations</Label>
                  <p className="text-sm text-muted-foreground">Enable smooth transitions and animations</p>
                </div>
                <Switch
                  id="showAnimations"
                  checked={settings.preferences.showAnimations}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showAnimations', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showTooltips">Show Tooltips</Label>
                  <p className="text-sm text-muted-foreground">Display helpful tooltips on hover</p>
                </div>
                <Switch
                  id="showTooltips"
                  checked={settings.preferences.showTooltips}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showTooltips', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showNotifications">Show Notifications</Label>
                  <p className="text-sm text-muted-foreground">Display system notifications</p>
                </div>
                <Switch
                  id="showNotifications"
                  checked={settings.preferences.showNotifications}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showNotifications', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showStatusIndicators">Status Indicators</Label>
                  <p className="text-sm text-muted-foreground">Show status indicators and badges</p>
                </div>
                <Switch
                  id="showStatusIndicators"
                  checked={settings.preferences.showStatusIndicators}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showStatusIndicators', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showProgressBars">Progress Bars</Label>
                  <p className="text-sm text-muted-foreground">Display progress indicators</p>
                </div>
                <Switch
                  id="showProgressBars"
                  checked={settings.preferences.showProgressBars}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showProgressBars', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showBreadcrumbs">Breadcrumbs</Label>
                  <p className="text-sm text-muted-foreground">Show navigation breadcrumbs</p>
                </div>
                <Switch
                  id="showBreadcrumbs"
                  checked={settings.preferences.showBreadcrumbs}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showBreadcrumbs', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="showSearchSuggestions">Search Suggestions</Label>
                  <p className="text-sm text-muted-foreground">Show search suggestions and autocomplete</p>
                </div>
                <Switch
                  id="showSearchSuggestions"
                  checked={settings.preferences.showSearchSuggestions}
                  onCheckedChange={(checked) => handleNestedSettingChange('preferences', 'showSearchSuggestions', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Accessibility */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Accessibility</span>
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
                  <Label htmlFor="highContrast">High Contrast</Label>
                  <p className="text-sm text-muted-foreground">Use high contrast colors</p>
                </div>
                <Switch
                  id="highContrast"
                  checked={settings.accessibility.highContrast}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'highContrast', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="reducedMotion">Reduced Motion</Label>
                  <p className="text-sm text-muted-foreground">Reduce animations and transitions</p>
                </div>
                <Switch
                  id="reducedMotion"
                  checked={settings.accessibility.reducedMotion}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'reducedMotion', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="largeText">Large Text</Label>
                  <p className="text-sm text-muted-foreground">Use larger text sizes</p>
                </div>
                <Switch
                  id="largeText"
                  checked={settings.accessibility.largeText}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'largeText', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="screenReader">Screen Reader</Label>
                  <p className="text-sm text-muted-foreground">Optimize for screen readers</p>
                </div>
                <Switch
                  id="screenReader"
                  checked={settings.accessibility.screenReader}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'screenReader', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="keyboardNavigation">Keyboard Navigation</Label>
                  <p className="text-sm text-muted-foreground">Enable keyboard navigation</p>
                </div>
                <Switch
                  id="keyboardNavigation"
                  checked={settings.accessibility.keyboardNavigation}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'keyboardNavigation', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="focusIndicators">Focus Indicators</Label>
                  <p className="text-sm text-muted-foreground">Show focus indicators</p>
                </div>
                <Switch
                  id="focusIndicators"
                  checked={settings.accessibility.focusIndicators}
                  onCheckedChange={(checked) => handleNestedSettingChange('accessibility', 'focusIndicators', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Advanced */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Advanced</span>
          </CardTitle>
          <CardDescription>
            Advanced customization and development options
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="enableDevTools">Developer Tools</Label>
                    <p className="text-sm text-muted-foreground">Enable development tools</p>
                  </div>
                  <Switch
                    id="enableDevTools"
                    checked={settings.advanced.enableDevTools}
                    onCheckedChange={(checked) => handleNestedSettingChange('advanced', 'enableDevTools', checked)}
                  />
                </div>
                
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="enableDebugMode">Debug Mode</Label>
                    <p className="text-sm text-muted-foreground">Enable debug logging</p>
                  </div>
                  <Switch
                    id="enableDebugMode"
                    checked={settings.advanced.enableDebugMode}
                    onCheckedChange={(checked) => handleNestedSettingChange('advanced', 'enableDebugMode', checked)}
                  />
                </div>
                
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="enablePerformanceMode">Performance Mode</Label>
                    <p className="text-sm text-muted-foreground">Optimize for performance</p>
                  </div>
                  <Switch
                    id="enablePerformanceMode"
                    checked={settings.advanced.enablePerformanceMode}
                    onCheckedChange={(checked) => handleNestedSettingChange('advanced', 'enablePerformanceMode', checked)}
                  />
                </div>
              </div>
            </div>
            
            <div>
              <Label htmlFor="customCSS">Custom CSS</Label>
              <Textarea
                id="customCSS"
                value={settings.advanced.customCSS}
                onChange={(e) => handleNestedSettingChange('advanced', 'customCSS', e.target.value)}
                placeholder="/* Custom CSS styles */"
                rows={6}
                className="font-mono text-sm"
              />
            </div>
            
            <div>
              <Label htmlFor="customJS">Custom JavaScript</Label>
              <Textarea
                id="customJS"
                value={settings.advanced.customJS}
                onChange={(e) => handleNestedSettingChange('advanced', 'customJS', e.target.value)}
                placeholder="// Custom JavaScript code"
                rows={6}
                className="font-mono text-sm"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Actions */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Save className="h-5 w-5" />
            <span>Actions</span>
          </CardTitle>
          <CardDescription>
            Save, reset, or export your theme settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center space-x-4">
            <Button onClick={handleSaveSettings} disabled={isLoading || !hasChanges}>
              <Save className="h-4 w-4 mr-2" />
              {isLoading ? 'Saving...' : 'Save Settings'}
            </Button>
            
            <Button variant="outline" onClick={handleResetSettings}>
              <RefreshCw className="h-4 w-4 mr-2" />
              Reset to Default
            </Button>
            
            <Button variant="outline">
              <Download className="h-4 w-4 mr-2" />
              Export Theme
            </Button>
            
            <Button variant="outline">
              <Upload className="h-4 w-4 mr-2" />
              Import Theme
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
