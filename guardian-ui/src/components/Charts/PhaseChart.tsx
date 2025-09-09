import React, { useMemo } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { useMetrics } from '@/store/live';

interface PhaseChartProps {
  serverId: string;
  className?: string;
}

export const PhaseChart: React.FC<PhaseChartProps> = React.memo(({ serverId, className = '' }) => {
  const metrics = useMetrics(serverId);
  
  // Memoize chart data with time window (last 120 seconds)
  const chartData = useMemo(() => {
    if (!metrics?.tickP95) return [];
    
    const now = Date.now();
    const timeWindow = 120000; // 2 minutes
    const cutoff = now - timeWindow;
    
    return metrics.tickP95
      .filter(point => point.timestamp > cutoff)
      .map(point => ({
        time: new Date(point.timestamp).toLocaleTimeString(),
        tickP95: point.value,
        timestamp: point.timestamp,
      }))
      .slice(-120); // Keep max 120 points
  }, [metrics?.tickP95]);

  // Memoize chart configuration
  const chartConfig = useMemo(() => ({
    data: chartData,
    margin: { top: 5, right: 30, left: 20, bottom: 5 },
  }), [chartData]);

  if (!chartData.length) {
    return (
      <div className={`panel p-4 ${className}`}>
        <h3 className="text-lg font-semibold mb-4">Tick Phase (P95)</h3>
        <div className="h-64 flex items-center justify-center text-muted-foreground">
          No tick phase data available
        </div>
      </div>
    );
  }

  return (
    <div className={`panel p-4 ${className}`}>
      <h3 className="text-lg font-semibold mb-4">Tick Phase (P95)</h3>
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
              label={{ value: 'ms', angle: -90, position: 'insideLeft' }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'hsl(220 12% 9%)',
                border: '1px solid hsl(218 12% 17%)',
                borderRadius: '8px',
                color: 'hsl(220 14% 92%)',
              }}
              labelStyle={{ color: 'hsl(220 14% 92%)' }}
              formatter={(value: number) => [`${value.toFixed(2)} ms`, 'Tick P95']}
            />
            <Line
              type="monotone"
              dataKey="tickP95"
              stroke="hsl(38 92% 50%)"
              strokeWidth={2}
              dot={false}
              activeDot={{ r: 4, fill: 'hsl(38 92% 50%)' }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
});

PhaseChart.displayName = 'PhaseChart';

export default PhaseChart;
