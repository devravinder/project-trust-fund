// Encode/decode TrustFund connection details for QR sharing.
// Format: a JSON object with a type tag so the scanner can validate it.

export interface ConnectionPayload {
  url: string
  token: string
}

const TAG = 'trustfund-conn'

export function encodeConnection(url: string, token: string): string {
  return JSON.stringify({ t: TAG, url, token })
}

export function decodeConnection(raw: string): ConnectionPayload {
  let obj: unknown
  try {
    obj = JSON.parse(raw)
  } catch {
    throw new Error('QR code is not valid TrustFund connection data')
  }
  if (
    typeof obj !== 'object' ||
    obj === null ||
    (obj as Record<string, unknown>).t !== TAG
  ) {
    throw new Error('QR code is not a TrustFund connection code')
  }
  const { url, token } = obj as Record<string, unknown>
  if (typeof url !== 'string' || typeof token !== 'string' || !url || !token) {
    throw new Error('QR code is missing url or token')
  }
  return { url, token }
}
