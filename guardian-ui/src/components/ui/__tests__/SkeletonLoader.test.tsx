import { render, screen } from '@testing-library/react'
import { SkeletonLoader, ServerCardSkeleton } from '../SkeletonLoader'

describe('SkeletonLoader', () => {
  it('renders default skeleton', () => {
    render(<SkeletonLoader />)
    
    const skeleton = screen.getByRole('generic')
    expect(skeleton).toHaveClass('animate-pulse')
  })

  it('renders multiple skeletons when count is provided', () => {
    render(<SkeletonLoader count={3} />)
    
    const skeletons = screen.getAllByTestId('skeleton-loader')
    expect(skeletons).toHaveLength(3)
  })

  it('renders card variant correctly', () => {
    render(<SkeletonLoader variant="card" />)
    
    const skeleton = screen.getByRole('generic')
    expect(skeleton).toHaveClass('animate-pulse')
  })

  it('renders table variant correctly', () => {
    render(<SkeletonLoader variant="table" />)
    
    const skeleton = screen.getByRole('generic')
    expect(skeleton).toHaveClass('animate-pulse')
  })

  it('renders chart variant correctly', () => {
    render(<SkeletonLoader variant="chart" />)
    
    const skeleton = screen.getByRole('generic')
    expect(skeleton).toHaveClass('animate-pulse')
  })

  it('renders list variant correctly', () => {
    render(<SkeletonLoader variant="list" />)
    
    const skeleton = screen.getByRole('generic')
    expect(skeleton).toHaveClass('animate-pulse')
  })

  it('applies custom className', () => {
    render(<SkeletonLoader className="custom-class" />)
    
    const skeleton = screen.getByTestId('skeleton-loader')
    expect(skeleton).toHaveClass('custom-class')
  })
})

describe('ServerCardSkeleton', () => {
  it('renders server card skeleton structure', () => {
    render(<ServerCardSkeleton />)
    
    const skeleton = screen.getByTestId('server-card-skeleton')
    expect(skeleton).toBeInTheDocument()
  })
})
