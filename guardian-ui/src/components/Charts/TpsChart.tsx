import React from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine
} from 'recharts';
import { Activity, TrendingUp, TrendingDown } from 'lucide-react';

interface TpsChartProps {
  data: Array<{
    timestamp: number;
    value: number;
  }>;
  className?: string;
}

export const TpsChart: React.FC<TpsChartProps> = ({
  data,
  className = ''
}) => {
  // Transform data for Recharts
  const chartData = data.map(point => ({
    time: new Date(point.timestamp).toLocaleTimeString(),
    timestamp: point.timestamp,
    tps: point.value
  }));

  // Calculate average TPS
  const averageTps = data.reduce((sum, point) => sum + point.value, 0) / data.length;
  
  // Calculate trend
  const firstHalf = data.slice(0, Math.floor(data.length / 2));
  const secondHalf = data.slice(Math.floor(data.length / 2));
  const firstHalfAvg = firstHalf.reduce((sum, point) => sum + point.value, 0) / firstHalf.length;
  const secondHalfAvg = secondHalf.reduce((sum, point) => sum + point.value, 0) / secondHalf.length;
  const trend = secondHalfAvg - firstHalfAvg;

  const getTpsColor = (tps: number) => {
    if (tps >= 19) return '#10b981'; // green
    if (tps >= 17) return '#f59e0b'; // yellow
    return '#ef4444'; // red
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
              style={{ backgroundColor: getTpsColor(data.tps) }}
            />
            <span className="text-sm">
              TPS: <span className="font-medium">{data.tps.toFixed(2)}</span>
            </span>
          </div>
        </div>
      );
    }
    return null;
  };

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Chart Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Activity className="h-4 w-4" />
          <span className="text-sm font-medium">TPS Performance</span>
        </div>
        
        <div className="flex items-center gap-4 text-sm">
          <div className="flex items-center gap-1">
            <span className="text-muted-foreground">Avg:</span>
            <span className="font-medium">{averageTps.toFixed(2)}</span>
          </div>
          
          <div className="flex items-center gap-1">
            {trend > 0 ? (
              <TrendingUp className="h-3 w-3 text-green-400" />
            ) : (
              <TrendingDown className="h-3 w-3 text-red-400" />
            )}
            <span className={`text-xs ${trend > 0 ? 'text-green-400' : 'text-red-400'}`}>
              {trend > 0 ? '+' : ''}{trend.toFixed(2)}
            </span>
          </div>
        </div>
      </div>

      {/* Chart */}
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis 
              dataKey="time" 
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
              axisLine={false}
            />
            <YAxis 
              domain={[15, 20]}
              stroke="#9ca3af"
              fontSize={12}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip content={<CustomTooltip />} />
            
            {/* Reference lines for TPS thresholds */}
            <ReferenceLine y={20} stroke="#10b981" strokeDasharray="2 2" strokeOpacity={0.5} />
            <ReferenceLine y={18} stroke="#f59e0b" strokeDasharray="2 2" strokeOpacity={0.5} />
            <ReferenceLine y={15} stroke="#ef4444" strokeDasharray="2 2" strokeOpacity={0.5} />
            
            <Line
              type="monotone"
              dataKey="tps"
              stroke="#3b82f6"
              strokeWidth={2}
              dot={false}
              activeDot={{ r: 4, fill: '#3b82f6' }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Legend */}
      <div className="flex items-center justify-center gap-6 text-xs">
        <div className="flex items-center gap-2">
          <div className="w-3 h-0.5 bg-green-500" />
          <span className="text-muted-foreground">Target (20 TPS)</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-0.5 bg-yellow-500" />
          <span className="text-muted-foreground">Warning (18 TPS)</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-0.5 bg-red-500" />
          <span className="text-muted-foreground">Critical (15 TPS)</span>
        </div>
      </div>
    </div>
  );
};

export default TpsChart;
