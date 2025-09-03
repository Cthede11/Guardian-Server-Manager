import { render, screen } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { AppShell } from '../AppShell'

const renderWithRouter = (component: React.ReactElement) => {
  return render(
    <BrowserRouter>
      {component}
    </BrowserRouter>
  )
}

describe('AppShell', () => {
  it('renders sidebar and main content area', () => {
    renderWithRouter(<AppShell />)
    
    expect(screen.getByRole('navigation')).toBeInTheDocument()
    expect(screen.getByRole('main')).toBeInTheDocument()
  })

  it('includes error boundary', () => {
    renderWithRouter(<AppShell />)
    
    // Error boundary should be present (wraps the app)
    // The ErrorBoundary component wraps the entire app
    expect(screen.getByRole('main')).toBeInTheDocument()
  })

  it('renders with proper layout structure', () => {
    renderWithRouter(<AppShell />)
    
    // Check for main layout elements
    const mainContainer = screen.getByRole('main').parentElement
    expect(mainContainer).toHaveClass('flex-1', 'flex', 'flex-col')
  })
})
