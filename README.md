DodoDB

A lightweight, in-memory key–value database and Pub/Sub microservice written in Rust.

Overview

DodoDB is a small and efficient in-memory database designed for local microservice architectures, edge applications, and rapid prototyping.
It provides:
	•	A simple REST API for setting, getting, deleting, and listing keys.
	•	Automatic JSON-based persistence (snapshots).
	•	Optional retention policies and cleanup loops.
	•	A built-in Pub/Sub system with webhook notifications.
	•	Cross-platform builds (Linux, macOS, Windows).

The architecture is intentionally minimal: a single executable, no external dependencies, and very low latency.

⸻

What is an In-Memory Database

An in-memory database stores all data directly in RAM instead of writing it to disk for each operation.
The advantages are:
	•	Very fast reads and writes.
	•	Low latency for real-time applications.
	•	Simple internal architecture.

DodoDB persists data to disk only periodically through snapshot files. This ensures durability while keeping runtime performance optimal.

In-memory databases are commonly used in caches, event systems, distributed queues, and microservices where state must be fast, simple, and local.

⸻

What is Pub/Sub

Pub/Sub (Publish/Subscribe) is an asynchronous communication model where:
	•	A publisher emits events.
	•	One or more subscribers are notified whenever an event occurs.
	•	Publishers do not need to know who the subscribers are.

This makes the system decoupled and scalable.

In DodoDB, a Pub/Sub event is generated whenever a key changes.
Subscribers register their interest in a specific key, and DodoDB sends a webhook notification each time that key is updated.

⸻

Pub/Sub in DodoDB

DodoDB implements Pub/Sub through a webhook-based mechanism.

How it Works
	1.	The client calls /pubsub/subscribe with:
	•	The key to watch
	•	A callback URL where the client can receive events
	2.	DodoDB stores the subscription in memory.
	3.	When the key is updated via /kv/<key>:
	•	The database writes the new value
	•	Pub/Sub builds an event payload containing:
	•	The key
	•	Event type (“update”)
	•	Old value
	•	New value
	•	Timestamp
	•	DodoDB sends an HTTP POST to the subscriber’s callback URL.
	4.	The subscriber processes the event and continues listening.

Payload example

{
  "key": "demo_value",
  "event": "update",
  "old_value": { "stage": 2 },
  "new_value": { "stage": 3 },
  "timestamp": "2025-01-01T12:00:00Z"
}

Why Webhooks

A webhook-based Pub/Sub model avoids maintaining persistent connections.
Each client implements a lightweight HTTP endpoint to receive events.
This works well across platforms and fits microservice environments.

⸻

REST API Summary

Key–Value Operations

Method	Path	Description
PUT	/kv/<key>	Store or overwrite JSON value
GET	/kv/<key>	Get the stored value
GET	/kv/<key>/exists	Check if a key exists
DELETE	/kv/<key>	Remove a key
GET	/kv	List all keys
GET	/kv/count	Count stored keys
POST	/kv/clear	Delete all keys

Pub/Sub Routes

Method	Path	Description
POST	/pubsub/subscribe	Register a webhook subscription
POST	/pubsub/unsubscribe	Remove a subscription

System Routes

Method	Path	Description
GET	/system/alive	Check server availability
GET	/system/version	Return configured server version


⸻

Cross-Platform Support

DodoDB runs on:
	•	macOS
	•	Linux
	•	Windows

It is written in Rust and does not require platform-specific libraries or services.

⸻

Architecture Summary
	•	Axum powers the HTTP server.
	•	Serde JSON is used for data serialization.
	•	Tokio handles concurrency and periodic tasks.
	•	Snapshot persistence saves a JSON dump at regular intervals.
	•	Pub/Sub uses webhook callbacks for cross-platform event propagation.

DodoDB is intentionally simple: all state is held in memory and guarded by thread-safe structures. Snapshot persistence ensures that data can be restored between restarts, making it suitable for small applications, prototypes, and local automation systems.

⸻

When to Use DodoDB

DodoDB is appropriate when:
	•	Ultra-fast, local state storage is needed.
	•	You want a simple distributed system built from microservices.
	•	You need webhook-based notifications for data changes.
	•	You prefer a standalone executable instead of a full database server.
	•	You want a database that runs identically on Windows, macOS, and Linux.

⸻

Andrea Panizzut - andrea.panizzut@gmail.com - December 2025








  
