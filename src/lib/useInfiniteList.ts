import { useCallback, useEffect, useRef, useState } from 'react'

const PAGE_SIZE = 25

/**
 * Infinite-scroll pagination over a page-fetching function.
 * `fetchPage(limit, offset)` must return one page of items (newest-first).
 * Re-runs from the start whenever `deps` change (e.g. search/filter).
 */
export function useInfiniteList<T>(
  fetchPage: (limit: number, offset: number) => Promise<T[]>,
  deps: unknown[],
  onError?: (e: unknown) => void,
) {
  const [items, setItems] = useState<T[]>([])
  const [loading, setLoading] = useState(true)
  const [loadingMore, setLoadingMore] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const offsetRef = useRef(0)
  const loadingRef = useRef(false)

  // Keep the latest fetchPage without forcing effect re-runs.
  const fetchRef = useRef(fetchPage)
  fetchRef.current = fetchPage

  const reset = useCallback(async () => {
    loadingRef.current = true
    setLoading(true)
    try {
      const first = await fetchRef.current(PAGE_SIZE, 0)
      setItems(first)
      offsetRef.current = first.length
      setHasMore(first.length === PAGE_SIZE)
    } catch (e) {
      onError?.(e)
    } finally {
      setLoading(false)
      loadingRef.current = false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [onError])

  const loadMore = useCallback(async () => {
    if (loadingRef.current || !hasMore) return
    loadingRef.current = true
    setLoadingMore(true)
    try {
      const next = await fetchRef.current(PAGE_SIZE, offsetRef.current)
      setItems((prev) => [...prev, ...next])
      offsetRef.current += next.length
      setHasMore(next.length === PAGE_SIZE)
    } catch (e) {
      onError?.(e)
    } finally {
      setLoadingMore(false)
      loadingRef.current = false
    }
  }, [hasMore, onError])

  // Reset when deps change (debounced search handled by caller).
  useEffect(() => {
    void reset()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps)

  return { items, loading, loadingMore, hasMore, loadMore, reload: reset }
}

export { PAGE_SIZE }
