import { useEffect, useRef, useState } from 'react'
import { Html5Qrcode } from 'html5-qrcode'

const REGION_ID = 'qr-scan-region'

export function QrScanner({
  onResult,
  onError,
}: {
  onResult: (text: string) => void
  onError?: (message: string) => void
}) {
  const scannerRef = useRef<Html5Qrcode | null>(null)
  const [starting, setStarting] = useState(true)
  const handledRef = useRef(false)

  useEffect(() => {
    let cancelled = false
    const scanner = new Html5Qrcode(REGION_ID)
    scannerRef.current = scanner

    Html5Qrcode.getCameras()
      .then((cameras) => {
        if (cancelled) return
        if (!cameras || cameras.length === 0) {
          onError?.('No camera found on this device.')
          setStarting(false)
          return
        }
        // Prefer a back camera if labelled.
        const back = cameras.find((c) => /back|rear|environment/i.test(c.label))
        const cameraId = back?.id ?? cameras[0]!.id
        return scanner
          .start(
            cameraId,
            { fps: 10, qrbox: 220 },
            (decoded) => {
              if (handledRef.current) return
              handledRef.current = true
              onResult(decoded)
            },
            () => {
              /* per-frame decode failure: ignore */
            },
          )
          .then(() => {
            if (!cancelled) setStarting(false)
          })
      })
      .catch((e) => {
        onError?.(
          `Camera access failed: ${e}. Grant camera permission or use manual entry.`,
        )
        setStarting(false)
      })

    return () => {
      cancelled = true
      const s = scannerRef.current
      if (s && s.isScanning) {
        s.stop()
          .then(() => s.clear())
          .catch(() => {
            /* ignore stop errors on unmount */
          })
      }
    }
  }, [onResult, onError])

  return (
    <div className="flex flex-col items-center gap-2">
      <div
        id={REGION_ID}
        className="w-full max-w-xs overflow-hidden rounded-md"
      />
      {starting && (
        <p className="text-xs text-muted-foreground">Starting camera…</p>
      )}
    </div>
  )
}
