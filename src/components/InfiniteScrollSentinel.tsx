import { useEffect, useRef } from 'react'

/**
 * Renders a sentinel element; calls `onReach` when it scrolls into view.
 * Works with any scroll container (uses the viewport/root by default, which
 * covers the app's scrollable <main>).
 */
export function InfiniteScrollSentinel({
  onReach,
  disabled,
}: {
  onReach: () => void
  disabled?: boolean
}) {
  const ref = useRef<HTMLDivElement>(null)
  const onReachRef = useRef(onReach)
  onReachRef.current = onReach

  useEffect(() => {
    if (disabled) return
    const el = ref.current
    if (!el) return
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) onReachRef.current()
      },
      { rootMargin: '200px' }, // prefetch a bit before fully visible
    )
    observer.observe(el)
    return () => observer.disconnect()
  }, [disabled])

  return <div ref={ref} aria-hidden className="h-1 w-full" />
}
