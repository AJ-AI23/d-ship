# Encryption and Receiver Access

This document describes how shipment data is encrypted, stored, and accessed by different roles (carrier, shipper, receiver).

---

## Data Segmentation

| Segment | Who Accesses | Storage | Mechanism |
|---------|--------------|---------|-----------|
| Tracking status, events | Carrier, Shipper, Receiver | Plaintext on-chain | `Tracker.getTrackingStatus(tracking_number)` |
| Full shipment record (addresses, PII) | Carrier, Shipper | Encrypted on-chain or off-chain | Client-side encrypt; key shared with carrier/shipper |
| Receiver-visible sensitive data | Receiver | Encrypted on-chain | Client-side encrypt; receiver gets key via email/SMS |

---

## Key Derivation

For encrypted payload stored per tracking number:

```
K = KDF(tracking_number, receiver_secret)
```

**Parameters:**
- `tracking_number`: The shipment tracking number (serial from Serial contract). Known to carrier, shipper, and receiver.
- `receiver_secret`: A secret sent to the receiver via a separate channel (email, SMS, printed on label, QR code).

**Recommended KDF:** PBKDF2 with SHA-256 (or scrypt). Example parameters:
- Iterations: 100,000+
- Salt: `tracking_number` (or a fixed salt per deployment)
- Output: 256-bit key for AES-256-GCM

**Frontend/off-chain implementation:**
1. Encrypt: `plaintext -> AES-256-GCM(K, plaintext)` before calling `createShipment` with `encrypted_payload`
2. Decrypt: Fetch ciphertext via `Tracker.getEncryptedPayload(tracking_number)`; user enters `receiver_secret`; derive K; decrypt in browser

---

## Receiver Secret Distribution

The receiver must receive `receiver_secret` through a channel separate from the tracking number. Options:

1. **Email/SMS**: Shipper sends `receiver_secret` to recipient's email/phone when shipment is created.
2. **Printed on label**: Include a short PIN or QR code on the physical label; QR decodes to `receiver_secret`.
3. **Secure link**: One-time link that displays `receiver_secret`; link expires after use.

The tracking number is often visible on the label or in notifications. The `receiver_secret` is the second factor that restricts decryption to the intended receiver.

---

## Flow Summary

1. **Shipment creation**: Shipper/carrier encrypts sensitive data with K, passes `encrypted_payload` to `createShipment`. Shipment contract forwards it to Tracker via `registerShipment`.
2. **Carrier/Shipper**: Have both tracking_number and receiver_secret from the creation flow; can decrypt at will.
3. **Receiver**: Receives (tracking_number, receiver_secret) via separate channels. Uses tracking website; enters both; frontend fetches ciphertext, derives K, decrypts, displays.

---

## Tracker Views

| View | Returns | Use |
|------|---------|-----|
| `getTrackingStatus(tracking_number)` | List of (event_type, timestamp, location) | Plaintext; anyone with tracking number |
| `getEncryptedPayload(tracking_number)` | Optional ciphertext | Off-chain decryption with K |
