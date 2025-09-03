import React from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell
} from 'recharts';
import { Zap, Clock, Layers, Network, Cpu } from 'lucide-react';

interface PhaseChartProps {
  data: Array<{
    timestamp: number;
    phase: string;
    time: number;
  }>;
  className?: string;
}

export const PhaseChart: React.FC<PhaseChartProps> = ({
  data,
  className = ''
}) => {
  // Group data by phase and calculate average time
  const phaseData = data.reduce((acc, point) => {
    if (!acc[point.phase]) {
      acc[point.phase] = [];
    }
    acc[point.phase].push(point.time);
    return acc;
  }, {} as Record<string, number[]>);

  // Calculate average time for each phase
  const chartData = Object.entries(phaseData).map(([phase, times]) => ({
    phase: phase.charAt(0).toUpperCase() + phase.slice(1),
    time: times.reduce((sum, time) => sum + time, 0) / times.length,
    count: times.length
  }));

  // Sort by time (highest first)
  chartData.sort((a, b) => b.time - a.time);

  const getPhaseColor = (phase: string) => {
    switch (phase.toLowerCase()) {
      case 'world':
        return '#10b981'; // green
      case 'entities':
        return '#3b82f6'; // blue
      case 'tileentities':
        return '#8b5cf6'; // purple
      case 'chunks':
        return '#f59e0b'; // yellow
      case 'network':
        return '#ef4444'; // red
      default:
        return '#6b7280'; // gray
    }
  };

  const getPhaseIcon = (phase: string) => {
    switch (phase.toLowerCase()) {
      case 'world':
        return <Layers className="h-4 w-4" />;
      case 'entities':
        return <Zap className="h-4 w-4" />;
      case 'tileentities':
        return <Cpu className="h-4 w-4" />;
      case 'chunks':
        return <Clock className="h-4 w-4" />;
      case 'network':
        return <Network className="h-4 w-4" />;
      default:
        return <Clock className="h-4 w-4" />;
    }
  };

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-background border border-border rounded-lg p-3 shadow-lg">
          <p className="text-sm font-medium">{label}</p>
          <div className="flex items-center gap-2 mt-1">
            <div 
              className="w-3 h-3 rounded-full" 
              style={{ backgroundColor: getPhaseColor(data.phase) }}
            />
            <span className="text-sm">
              Avg Time: <span className="font-medium">{data.time.toFixed(2)}ms</span>
            </span>
          </div>
          <div className="text-xs text-muted-foreground mt-1">
            Samples: {data.count}
          </div>
        </div>
      );
    }
    return null;
  };

  // Calculate total tick time
  const totalTime = chartData.reduce((sum, phase) => sum + phase.time, 0);

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Chart Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Zap className="h-4 w-4" />
          <span className="text-sm font-medium">Tick Phase Breakdown</span>
        </div>
        
        <div className="text-sm">
          <span className="text-muted-foreground">Total: </span>
          <span className="font-medium">{totalTime.toFixed(1)}ms</span>
        </div>
      </div>

      {/* Chart */}
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis 
              dataKey="phase" 
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
              axisLine={false}
            />
            <YAxis 
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
              axisLine={false}
              label={{ value: 'Time (ms)', angle: -90, position: 'insideLeft' }}
            />
            <Tooltip content={<CustomTooltip />} />
            <Bar dataKey="time" radius={[2, 2, 0, 0]}>
              {chartData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={getPhaseColor(entry.phase)} />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* Phase Details */}
      <div className="grid grid-cols-2 gap-2">
        {chartData.map((phase) => (
          <div key={phase.phase} className="flex items-center justify-between p-2 bg-muted/50 rounded">
            <div className="flex items-center gap-2">
              {getPhaseIcon(phase.phase)}
              <span className="text-sm font-medium">{phase.phase}</span>
            </div>
            <div className="text-right">
              <div className="text-sm font-medium">{phase.time.toFixed(1)}ms</div>
              <div className="text-xs text-muted-foreground">
                {((phase.time / totalTime) * 100).toFixed(1)}%
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Legend */}
      <div className="flex items-center justify-center gap-4 text-xs">
        {chartData.map((phase) => (
          <div key={phase.phase} className="flex items-center gap-1">
            <div 
              className="w-3 h-3 rounded-full" 
              style={{ backgroundColor: getPhaseColor(phase.phase) }}
            />
            <span className="text-muted-foreground">{phase.phase}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PhaseChart;
