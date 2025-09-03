import { render, screen } from '@testing-library/react'
import { StatusPill } from '../StatusPill'

describe('StatusPill', () => {
  it('renders running status correctly', () => {
    render(<StatusPill status="running" />)
    
    expect(screen.getByText('Running')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toBeInTheDocument()
  })

  it('renders stopped status correctly', () => {
    render(<StatusPill status="stopped" />)
    
    expect(screen.getByText('Stopped')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toBeInTheDocument()
  })

  it('renders starting status with animation', () => {
    render(<StatusPill status="starting" />)
    
    expect(screen.getByText('Starting')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toBeInTheDocument()
  })

  it('renders stopping status with animation', () => {
    render(<StatusPill status="stopping" />)
    
    expect(screen.getByText('Stopping')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toBeInTheDocument()
  })

  it('hides icon when showIcon is false', () => {
    render(<StatusPill status="running" showIcon={false} />)
    
    expect(screen.getByText('Running')).toBeInTheDocument()
    expect(screen.getByTestId('status-pill')).toBeInTheDocument()
  })

  it('applies custom className', () => {
    render(<StatusPill status="running" className="custom-class" />)
    
    const badge = screen.getByText('Running').closest('div')
    expect(badge).toHaveClass('custom-class')
  })
})
