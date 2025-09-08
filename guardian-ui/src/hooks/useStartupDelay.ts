import { useState, useEffect } from 'react';

export const useStartupDelay = (delay: number = 1000) => {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsReady(true);
    }, delay);

    return () => clearTimeout(timer);
  }, [delay]);

  return isReady;
};

export default useStartupDelay;
