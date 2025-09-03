import { render, screen, fireEvent } from '@testing-library/react'
import { EmptyState, NoServersEmptyState } from '../EmptyState'
import { Server } from 'lucide-react'

describe('EmptyState', () => {
  it('renders title and description', () => {
    render(
      <EmptyState
        title="No data found"
        description="Try refreshing the page"
      />
    )
    
    expect(screen.getByText('No data found')).toBeInTheDocument()
    expect(screen.getByText('Try refreshing the page')).toBeInTheDocument()
  })

  it('renders action button when provided', () => {
    const mockAction = vi.fn()
    
    render(
      <EmptyState
        title="No data found"
        action={{
          label: 'Refresh',
          onClick: mockAction
        }}
      />
    )
    
    const button = screen.getByText('Refresh')
    expect(button).toBeInTheDocument()
    
    fireEvent.click(button)
    expect(mockAction).toHaveBeenCalled()
  })

  it('renders custom icon when provided', () => {
    render(
      <EmptyState
        title="No servers"
        icon={<Server className="h-12 w-12" />}
      />
    )
    
    expect(screen.getByRole('img', { hidden: true })).toBeInTheDocument()
  })

  it('renders secondary action when provided', () => {
    const mockSecondaryAction = vi.fn()
    
    render(
      <EmptyState
        title="No data"
        action={{
          label: 'Primary',
          onClick: vi.fn()
        }}
        secondaryAction={{
          label: 'Secondary',
          onClick: mockSecondaryAction
        }}
      />
    )
    
    const secondaryButton = screen.getByText('Secondary')
    expect(secondaryButton).toBeInTheDocument()
    
    fireEvent.click(secondaryButton)
    expect(mockSecondaryAction).toHaveBeenCalled()
  })

  it('applies different sizes correctly', () => {
    const { rerender } = render(
      <EmptyState
        title="Small"
        size="sm"
      />
    )
    
    expect(screen.getByText('Small')).toHaveClass('text-lg')
    
    rerender(
      <EmptyState
        title="Large"
        size="lg"
      />
    )
    
    expect(screen.getByText('Large')).toHaveClass('text-2xl')
  })
})

describe('NoServersEmptyState', () => {
  it('renders with correct content and action', () => {
    const mockCreateServer = vi.fn()
    
    render(<NoServersEmptyState onCreateServer={mockCreateServer} />)
    
    expect(screen.getByText('No servers found')).toBeInTheDocument()
    expect(screen.getByText('Create Server')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Create Server'))
    expect(mockCreateServer).toHaveBeenCalled()
  })
})
