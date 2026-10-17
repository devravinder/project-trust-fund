import { useEffect, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'

/**
 * Virtualized vertical list — renders only the rows in view (plus a small
 * overscan), so lists with thousands of items stay fast and light on memory.
 * It is its own scroll container (fills the available height), and calls
 * `onReachEnd` as the user nears the bottom to drive infinite pagination.
 */
export function VirtualList<T>({
  items,
  estimateSize = 76,
  gap = 8,
  overscan = 8,
  onReachEnd,
  hasMore,
  footer,
  renderItem,
  getKey,
}: {
  items: T[]
  estimateSize?: number
  gap?: number
  overscan?: number
  onReachEnd?: () => void
  hasMore?: boolean
  footer?: React.ReactNode
  renderItem: (item: T, index: number) => React.ReactNode
  getKey: (item: T, index: number) => string
}) {
  const parentRef = useRef<HTMLDivElement>(null)
  const onReachRef = useRef(onReachEnd)
  onReachRef.current = onReachEnd

  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => estimateSize + gap,
    overscan,
  })

  // Trigger pagination when the last virtual row is rendered.
  const virtualItems = rowVirtualizer.getVirtualItems()
  const lastIndex = virtualItems[virtualItems.length - 1]?.index ?? 0
  useEffect(() => {
    if (hasMore && lastIndex >= items.length - 1 && items.length > 0) {
      onReachRef.current?.()
    }
  }, [lastIndex, items.length, hasMore])

  return (
    <div ref={parentRef} className="h-full overflow-y-auto">
      <div
        style={{
          height: rowVirtualizer.getTotalSize(),
          position: 'relative',
          width: '100%',
        }}
      >
        {virtualItems.map((v) => (
          <div
            key={getKey(items[v.index]!, v.index)}
            ref={rowVirtualizer.measureElement}
            data-index={v.index}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              transform: `translateY(${v.start}px)`,
              paddingBottom: gap,
            }}
          >
            {renderItem(items[v.index]!, v.index)}
          </div>
        ))}
      </div>
      {footer}
    </div>
  )
}
