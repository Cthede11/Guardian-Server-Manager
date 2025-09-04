import React, { useMemo } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { useMetrics } from '@/store/live';

interface HeapChartProps {
  serverId: string;
  className?: string;
}

export const HeapChart: React.FC<HeapChartProps> = React.memo(({ serverId, className = '' }) => {
  const metrics = useMetrics(serverId);
  
  // Memoize chart data with time window (last 120 seconds)
  const chartData = useMemo(() => {
    if (!metrics?.heap) return [];
    
    const now = Date.now();
    const timeWindow = 120000; // 2 minutes
    const cutoff = now - timeWindow;
    
    return metrics.heap
      .filter(point => point.timestamp > cutoff)
      .map(point => ({
        time: new Date(point.timestamp).toLocaleTimeString(),
        heap: Math.round(point.value / 1024 / 1024), // Convert to MB
        timestamp: point.timestamp,
      }))
      .slice(-120); // Keep max 120 points
  }, [metrics?.heap]);

  // Memoize chart configuration
  const chartConfig = useMemo(() => ({
    data: chartData,
    margin: { top: 5, right: 30, left: 20, bottom: 5 },
  }), [chartData]);

  if (!chartData.length) {
    return (
      <div className={`panel p-4 ${className}`}>
        <h3 className="text-lg font-semibold mb-4">Heap Usage</h3>
        <div className="h-64 flex items-center justify-center text-muted-foreground">
          No heap data available
        </div>
      </div>
    );
  }

  return (
    <div className={`panel p-4 ${className}`}>
      <h3 className="text-lg font-semibold mb-4">Heap Usage</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart {...chartConfig}>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(220 8% 64%)" />
            <XAxis 
              dataKey="time" 
              stroke="hsl(220 14% 92%)"
              fontSize={12}
              tick={{ fill: 'hsl(220 8% 64%)' }}
            />
            <YAxis 
              stroke="hsl(220 14% 92%)"
              fontSize={12}
              tick={{ fill: 'hsl(220 8% 64%)' }}
              label={{ value: 'MB', angle: -90, position: 'insideLeft' }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'hsl(220 12% 9%)',
                border: '1px solid hsl(218 12% 17%)',
                borderRadius: '8px',
                color: 'hsl(220 14% 92%)',
              }}
              labelStyle={{ color: 'hsl(220 14% 92%)' }}
              formatter={(value: number) => [`${value} MB`, 'Heap Usage']}
            />
            <Line
              type="monotone"
              dataKey="heap"
              stroke="hsl(142 76% 36%)"
              strokeWidth={2}
              dot={false}
              activeDot={{ r: 4, fill: 'hsl(142 76% 36%)' }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
});

HeapChart.displayName = 'HeapChart';

export default HeapChart;
