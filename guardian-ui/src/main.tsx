import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { TestComponent } from './TestComponent';
import './index.css';

// Debug: Check if root element exists
const rootElement = document.getElementById('root');
console.log('Root element:', rootElement);

if (!rootElement) {
  console.error('Root element not found!');
} else {
  console.log('Root element found, rendering app...');
  
  try {
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <div>
          <TestComponent />
          <RouterProvider router={router} />
        </div>
      </React.StrictMode>
    );
  } catch (error) {
    console.error('Error rendering app:', error);
    // Fallback to simple component
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <TestComponent />
      </React.StrictMode>
    );
  }
}
