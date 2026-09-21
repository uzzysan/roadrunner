# Stripe webhook (legacy MVP)

Set `STRIPE_WEBHOOK_SECRET` to the signing secret (`whsec_...`) of the **specific** Stripe webhook endpoint (the Stripe CLI forwarding secret is different from the Dashboard endpoint secret). Set `STRIPE_WEBHOOK_API_VERSION` to that endpoint's exact snapshot event API version, including any suffix. Keep both values in a secret manager; never commit them or log headers/body.

Subscribe the endpoint at `POST /webhooks/stripe` to `payment_intent.succeeded` and `payment_intent.payment_failed`. The server verifies the `Stripe-Signature` HMAC over the unmodified request body and rejects timestamps outside a 300-second window. Missing/invalid signatures and malformed event data receive 400. Missing configuration, mismatched API version, missing local PaymentIntent, or database failures receive 500 so Stripe can retry. Unknown event types are acknowledged after storing their event ID; they do not change payments.

Apply the SQL migrations before accepting webhooks. The `stripe_webhook_events` primary key and the payment update share a single database transaction. This makes duplicate deliveries safe even when concurrent; terminal states cannot be changed by a late `payment_intent.payment_failed`. A verified event arriving before its local payment row is rolled back and retried. Monitor recurring 5xx deliveries, especially version mismatches. Stripe does not guarantee event order.

To validate with a database, configure a disposable Postgres database with the normal schema/migrations and send the same signed event concurrently. Confirm one event row, one status transition, 2xx responses for both deliveries, and no `succeeded` → `failed` transition after a later signed failure event. Unit tests cover signature validation without Postgres.
