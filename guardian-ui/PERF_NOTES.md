# Guardian UI Performance Notes

## Performance Optimizations Implemented

### 1. Route-Level Code Splitting
- **Implementation**: All server tab pages (`/servers/:id/*`) are lazy-loaded using `React.lazy()` and `Suspense`
- **Benefit**: Reduces initial bundle size and improves Time to Interactive (TTI)
- **Target**: Tab switch < 100ms (cold) / < 16ms (warm)

### 2. Virtualization for Large Lists
- **Console Stream**: Uses `@tanstack/react-virtual` with 5000-line rolling buffer
- **Players Table**: Virtualized with 80px row height, handles 2k+ players smoothly
- **Benefit**: Constant memory usage regardless of list size
- **Target**: 60 FPS scrolling with 1k+ items

### 3. Batched State Updates
- **Implementation**: Socket/SSE events are batched using `requestAnimationFrame`
- **Store**: Zustand stores use narrow selectors with `shallow` comparison
- **Benefit**: Prevents re-render storms from high-frequency updates
- **Target**: < 3ms work per update cycle

### 4. Memoization Strategy
- **Components**: Heavy components wrapped in `React.memo`
- **Selectors**: Zustand selectors use `shallow` comparison
- **Filters**: Search/filter operations are memoized with `useMemo`
- **Benefit**: Prevents unnecessary re-renders

### 5. Time-Series Data Management
- **Charts**: Limited to 120 data points (2 minutes at 1Hz)
- **Console**: Rolling buffer of 5000 lines
- **Metrics**: Automatic cleanup of old data points
- **Benefit**: Bounded memory usage for real-time data

## Performance Budget

### Acceptable Metrics
- **Time to Interactive (dev)**: < 1s after Vite boots
- **Tab Switch**: < 100ms (cold) / < 16ms (warm)
- **Console Streaming**: 1k lines/minute without frame drops
- **Players List**: Smooth at 2k rows with actions
- **Chart Updates**: ≤ 1 commit per 1s tick, < 3ms work per update

### Monitoring
- Use Chrome DevTools Performance tab
- Watch for "Scripting" bars > 16ms
- Monitor memory usage in Task Manager
- Profile with React DevTools Profiler

## Profiling Guide

### Chrome DevTools
1. Open Performance tab
2. Start recording
3. Perform actions (scroll, switch tabs, etc.)
4. Stop recording
5. Look for:
   - Long tasks (> 16ms)
   - Excessive re-renders
   - Memory leaks

### React DevTools
1. Install React DevTools extension
2. Use Profiler tab
3. Record component renders
4. Look for:
   - Unnecessary re-renders
   - Expensive components
   - Poor memoization

### Why Did You Render (Dev Only)
```bash
npm install @welldone-software/why-did-you-render
```

Add to main.tsx:
```typescript
if (import.meta.env.DEV) {
  import('@welldone-software/why-did-you-render').then(whyDidYouRender => {
    whyDidYouRender.default(React, {
      trackAllPureComponents: true,
    });
  });
}
```

## Common Performance Issues

### 1. Re-render Storms
**Symptoms**: UI freezes during high-frequency updates
**Solution**: 
- Use `shallow` comparison in Zustand selectors
- Batch updates with `requestAnimationFrame`
- Memoize expensive computations

### 2. Memory Leaks
**Symptoms**: Memory usage grows over time
**Solution**:
- Clean up intervals/timeouts in `useEffect`
- Limit data retention (rolling buffers)
- Use `AbortController` for cancelled requests

### 3. Large Bundle Size
**Symptoms**: Slow initial load
**Solution**:
- Lazy load routes with `React.lazy`
- Code split by feature
- Tree shake unused dependencies

### 4. Expensive Renders
**Symptoms**: Janky animations, slow interactions
**Solution**:
- Virtualize large lists
- Memoize components with `React.memo`
- Use `useCallback` for event handlers

## Performance Testing

### Automated Tests
```bash
# Run performance tests
npm run test:perf

# Lighthouse CI
npm run lighthouse
```

### Manual Testing
1. **Console Stream**: Send 1000+ messages rapidly
2. **Players List**: Load 2000+ players
3. **Tab Switching**: Switch between tabs rapidly
4. **Memory**: Leave app running for 30+ minutes

## Optimization Checklist

- [ ] Routes are lazy-loaded
- [ ] Large lists are virtualized
- [ ] State updates are batched
- [ ] Selectors use shallow comparison
- [ ] Components are memoized where appropriate
- [ ] Time-series data is bounded
- [ ] Memory leaks are prevented
- [ ] Bundle size is optimized
- [ ] Performance budget is met

## Future Optimizations

1. **Web Workers**: Move heavy JSON parsing off main thread
2. **Service Workers**: Cache static assets and API responses
3. **Streaming SSR**: Server-side rendering for faster initial load
4. **Micro-frontends**: Split into smaller, independent apps
5. **CDN**: Serve static assets from edge locations
