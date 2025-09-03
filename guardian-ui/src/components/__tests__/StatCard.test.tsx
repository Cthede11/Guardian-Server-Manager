import { render, screen } from '@testing-library/react'
import { StatCard } from '../StatCard'
import { Activity } from 'lucide-react'

describe('StatCard', () => {
  it('renders title, value, and subtitle', () => {
    render(
      <StatCard
        title="TPS"
        value="20.0"
        subtitle="Ticks per second"
        icon={<Activity className="h-4 w-4" />}
      />
    )
    
    expect(screen.getByText('TPS')).toBeInTheDocument()
    expect(screen.getByText('20.0')).toBeInTheDocument()
    expect(screen.getByText('Ticks per second')).toBeInTheDocument()
  })

  it('renders trend indicator when provided', () => {
    render(
      <StatCard
        title="TPS"
        value="20.0"
        subtitle="Ticks per second"
        trend={{ value: 5, isPositive: true }}
      />
    )
    
    expect(screen.getByText('+5%')).toBeInTheDocument()
  })

  it('renders negative trend correctly', () => {
    render(
      <StatCard
        title="TPS"
        value="15.0"
        trend={{ value: 10, isPositive: false }}
      />
    )
    
    expect(screen.getByText('-10%')).toBeInTheDocument()
  })

  it('renders without subtitle when not provided', () => {
    render(
      <StatCard
        title="Players"
        value="5"
      />
    )
    
    expect(screen.getByText('Players')).toBeInTheDocument()
    expect(screen.getByText('5')).toBeInTheDocument()
    expect(screen.queryByText('Ticks per second')).not.toBeInTheDocument()
  })

  it('renders without icon when not provided', () => {
    render(
      <StatCard
        title="Memory"
        value="512MB"
      />
    )
    
    expect(screen.getByText('Memory')).toBeInTheDocument()
    expect(screen.getByText('512MB')).toBeInTheDocument()
  })

  it('applies custom className', () => {
    render(
      <StatCard
        title="Test"
        value="123"
        className="custom-class"
      />
    )
    
    const card = screen.getByText('Test').closest('.server-card')
    expect(card).toHaveClass('custom-class')
  })
})
