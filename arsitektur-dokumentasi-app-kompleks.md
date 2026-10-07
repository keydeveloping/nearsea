# Dokumentasi yang Dibutuhkan untuk Aplikasi Sangat Kompleks

Kalau targetnya **app yang benar-benar kompleks** dan nanti dikerjakan agent/AI, PRD saja biasanya belum cukup. Yang dibutuhkan adalah satu “source of truth” yang memecah produk → UX → arsitektur → data → API → implementasi → testing.

## Struktur yang direkomendasikan

```text
project/
├── README.md
├── AGENTS.md
│
├── docs/
│   ├── 00-project-overview.md
│   ├── 01-PRD.md
│   ├── 02-product-requirements.md
│   ├── 03-user-flows.md
│   ├── 04-ux-ui-spec.md
│   │
│   ├── architecture/
│   │   ├── system-architecture.md
│   │   ├── frontend-architecture.md
│   │   ├── backend-architecture.md
│   │   ├── infrastructure.md
│   │   └── tech-stack.md
│   │
│   ├── database/
│   │   ├── database-schema.md
│   │   ├── data-model.md
│   │   ├── migrations.md
│   │   └── seed-data.md
│   │
│   ├── api/
│   │   ├── api-overview.md
│   │   ├── authentication.md
│   │   ├── endpoints.md
│   │   └── webhooks.md
│   │
│   ├── features/
│   │   ├── auth.md
│   │   ├── users.md
│   │   ├── marketplace.md
│   │   ├── payments.md
│   │   └── notifications.md
│   │
│   ├── security/
│   │   ├── security.md
│   │   ├── permissions.md
│   │   └── threat-model.md
│   │
│   ├── testing/
│   │   ├── testing-strategy.md
│   │   ├── test-cases.md
│   │   └── acceptance-criteria.md
│   │
│   ├── deployment/
│   │   ├── environments.md
│   │   ├── deployment.md
│   │   ├── monitoring.md
│   │   └── disaster-recovery.md
│   │
│   └── decisions/
│       ├── ADR-001.md
│       ├── ADR-002.md
│       └── ...
│
└── tasks/
    ├── backlog.md
    ├── milestones.md
    └── implementation-plan.md
```

---

## 1. `README.md`

Ini adalah pintu masuk proyek.

Isinya jangan terlalu dalam.

Contoh:

```md
# Project Name

## Description
Apa aplikasi ini?

## Core Purpose
Masalah apa yang diselesaikan?

## Tech Stack
- Next.js
- Node.js
- PostgreSQL
- Redis
- S3
- Docker

## Repository Structure
Penjelasan folder utama.

## Development
Cara menjalankan project.

## Documentation
Link ke semua dokumen penting.
```

Tujuannya supaya manusia maupun agent bisa tahu **proyek ini sebenarnya apa** dalam waktu singkat.

---

## 2. `AGENTS.md`

Kalau sering pakai Claude Code / Codex / agent lain, ini justru sangat penting.

Isinya aturan kerja agent.

Misalnya:

```md
# AGENTS.md

## Project Rules

- Use TypeScript.
- Never use `any` unless explicitly justified.
- Follow feature-based architecture.
- All API endpoints require validation.
- Every new feature must include tests.
- Never directly modify production configuration.
- Database changes require migrations.

## Coding Style

...

## Git Rules

...

## Testing Rules

...

## Important Architecture Decisions

...

## Things Agents Must NOT Do

...
```

Jadi agent tidak cuma tahu **apa yang harus dibuat**, tetapi juga **bagaimana harus mengerjakannya**.

---

## 3. `01-PRD.md`

Ini dokumen **product-level**.

PRD menjawab:

> "Kita sebenarnya sedang membangun apa dan kenapa?"

Minimal berisi:

```md
# Product Requirements Document

## 1. Product Overview

## 2. Problem Statement

## 3. Goals

## 4. Non-Goals

## 5. Target Users

## 6. User Personas

## 7. Core Use Cases

## 8. Features

## 9. Functional Requirements

## 10. Non-Functional Requirements

## 11. User Stories

## 12. Acceptance Criteria

## 13. Business Rules

## 14. Constraints

## 15. Dependencies

## 16. Success Metrics

## 17. Future Scope
```

Contohnya:

```md
### Feature: NFT Listing

Users can list an NFT for sale.

Requirements:

- User must own the NFT.
- NFT must not already be listed.
- User must specify price.
- Price must be greater than 0.
- Listing transaction must be confirmed.
```

PRD menjelaskan **behavior bisnis**, bukan implementasi kode.

---

## 4. Product Requirements

Ini lebih detail dari PRD.

Misalnya PRD bilang:

> User dapat membeli NFT.

Product requirements menjabarkan:

```text
BUY NFT

1. User membuka detail NFT
2. Sistem memeriksa status listing
3. Sistem menampilkan harga
4. User klik Buy
5. Sistem meminta konfirmasi
6. Wallet melakukan signing
7. Transaction dikirim
8. Sistem menunggu confirmation
9. Ownership berubah
10. Listing menjadi SOLD
11. Buyer mendapat NFT
12. Seller mendapat payment
```

Ini sudah menjadi **behavior specification**.

---

## 5. `user-flows.md`

Ini menjelaskan perjalanan user.

Misalnya:

```text
Login

Landing Page
     ↓
Login
     ↓
Connect Wallet
     ↓
Wallet Signature
     ↓
Authentication
     ↓
Dashboard
```

Untuk sistem kompleks bisa dibuat:

```text
User Registration
User Login
Password Reset
Checkout
Payment
Refund
NFT Minting
NFT Listing
NFT Purchase
Admin Moderation
etc.
```

Ini sangat membantu UI designer, developer, QA, dan AI agent.

---

## 6. `ux-ui-spec.md`

Ini menjelaskan **apa yang user lihat dan lakukan**.

Misalnya:

```md
## Dashboard

### Header

Contains:
- Logo
- Search
- Notifications
- Profile
- Wallet

### Sidebar

- Dashboard
- Explore
- Collection
- Activity
- Settings

### Main Content

...
```

Bisa ditambah:

```text
Desktop
Tablet
Mobile

Responsive behavior

Loading state
Empty state
Error state
Success state
Skeleton state
Disabled state
```

Untuk app kompleks ini penting banget.

---

## 7. Architecture Documentation

Sekarang masuk ke **technical layer**.

### `system-architecture.md`

Menjawab:

> Sistem ini terdiri dari apa saja?

Contoh:

```text
                    ┌──────────────┐
                    │   Frontend   │
                    │   Next.js    │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ API Gateway  │
                    └──────┬───────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
       Auth Service   User Service   Marketplace
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                    PostgreSQL
```

Lalu jelaskan setiap component.

---

## 8. `tech-stack.md`

Jangan cuma bilang:

```text
Next.js
Node
Postgres
Redis
```

Tetapi:

```md
## Frontend

Framework:
Next.js 16

Language:
TypeScript

State:
Zustand

Server State:
TanStack Query

Styling:
Tailwind

## Backend

Runtime:
Node.js

Framework:
Fastify

Database:
PostgreSQL

Cache:
Redis
```

Agent akan jauh lebih konsisten kalau keputusan teknologi sudah ditetapkan.

---

## 9. Database Documentation

Ini salah satu yang paling sering dilupakan.

Minimal:

```text
database/
├── schema.md
├── data-model.md
├── relationships.md
└── migrations.md
```

Contoh:

```text
User
 │
 ├── Wallet
 │
 ├── Order
 │
 ├── Listing
 │
 └── Notification
```

Kemudian detail:

```text
users

id
email
username
password_hash
created_at
updated_at
```

Dan relationship:

```text
User 1 ─── N Orders
User 1 ─── N Listings
User 1 ─── N Notifications
```

Untuk aplikasi kompleks, **database schema sebenarnya adalah bagian inti dari architecture**.

---

## 10. API Specification

Misalnya:

```text
GET    /api/users/:id
POST   /api/users
PATCH  /api/users/:id
DELETE /api/users/:id
```

Tetapi dokumentasinya lebih detail:

```md
POST /api/listings

Auth:
Required

Request:

{
  "assetId": "uuid",
  "price": "1.5"
}

Response:

{
  "id": "uuid",
  "status": "ACTIVE"
}

Errors:

400 INVALID_PRICE
401 UNAUTHORIZED
403 NOT_OWNER
409 ALREADY_LISTED
```

Untuk app besar, API contract sangat penting karena frontend/backend bisa dikembangkan secara paralel.

---

## 11. Feature Specification

Ini menurut saya **sangat penting untuk AI coding**.

Jangan membuat agent membaca 200 halaman PRD untuk mengerjakan satu fitur.

Bikin:

```text
features/
├── auth.md
├── profile.md
├── marketplace.md
├── checkout.md
├── payment.md
└── notification.md
```

Misalnya `checkout.md`:

```md
# Checkout

## Objective

Allow users to purchase an item.

## Preconditions

- User authenticated
- Product available
- Payment method available

## Flow

...

## UI

...

## API

...

## Database

...

## Error Cases

...

## Security

...

## Acceptance Criteria

...
```

Dengan begini agent bisa fokus ke satu feature.

---

## 12. Security Documentation

Untuk aplikasi kompleks jangan dianggap belakangan.

Minimal:

```text
Authentication
Authorization
Roles
Permissions
Session
Token handling
Rate limiting
Input validation
File upload security
Encryption
Secrets management
Audit logs
CSRF
XSS
SQL Injection
SSRF
etc.
```

Contohnya:

```text
ROLE

ADMIN
MODERATOR
SELLER
BUYER
GUEST
```

Dengan permission:

```text
SELLER
├── create_listing
├── edit_own_listing
├── cancel_own_listing
└── view_orders

ADMIN
├── manage_users
├── moderate_content
├── manage_system
└── view_audit_logs
```

---

## 13. Error & Edge Cases

Ini sering banget dilupakan manusia dan AI.

Misalnya:

```text
What happens if:

- payment fails?
- payment succeeds but webhook fails?
- database goes down?
- request times out?
- user refreshes during checkout?
- duplicate request happens?
- transaction is submitted twice?
- wallet disconnects?
- user loses internet?
- stock becomes zero?
```

Dokumen:

```md
## Error Handling

### Payment Timeout

Expected behavior:
...

User message:
...

System action:
...

Retry policy:
...
```

Semakin kompleks aplikasinya, semakin penting bagian ini.

---

## 14. Testing Documentation

Minimal:

```text
Unit Test
Integration Test
API Test
E2E Test
Security Test
Performance Test
Regression Test
```

Bahkan buat acceptance criteria:

```md
## Acceptance Criteria

Given:
User owns NFT

When:
User creates listing

Then:
Listing appears in marketplace

And:
NFT cannot be listed again
```

Format seperti ini sangat bagus untuk QA **dan AI agent**.

---

## 15. Deployment Documentation

Ini menjawab:

> "Bagaimana aplikasi hidup di production?"

Misalnya:

```text
Development
     ↓
Staging
     ↓
Production
```

Dokumen:

```text
Environment variables
Docker
CI/CD
Database migration
Backup
Monitoring
Logging
Rollback
Domain
SSL
Cloud infrastructure
Scaling
```

---

## 16. Observability

Aplikasi besar tidak cukup hanya "jalan".

Perlu tahu:

```text
Logs
Metrics
Tracing
Error tracking
Health checks
Alerts
Audit logs
```

Contoh:

```text
/api/orders
Average latency
Error rate
Requests/minute
5xx rate
```

---

## 17. ADR — Architecture Decision Records

Ini underrated tapi **sangat berguna**.

Misalnya:

```text
ADR-001-use-postgresql.md
ADR-002-use-redis.md
ADR-003-monorepo.md
ADR-004-auth-with-jwt.md
```

Contoh:

```md
# ADR-004: Authentication Strategy

## Decision

Use HTTP-only session cookies.

## Why

...

## Alternatives

JWT
OAuth-only
...

## Consequences

...
```

Jadi 6 bulan kemudian kamu tidak bertanya:

> "Kenapa kita pakai ini?"

---

## 18. Implementation Plan

Ini menjembatani dokumen dengan coding.

Misalnya:

```text
Phase 1
├── Setup repository
├── Setup database
├── Authentication
└── User system

Phase 2
├── Marketplace
├── Listing
└── Search

Phase 3
├── Checkout
├── Payment
└── Orders
```

Kemudian dipecah sampai task:

```text
TASK-001
Setup PostgreSQL

TASK-002
Create users table

TASK-003
Create auth service

TASK-004
Create login endpoint

TASK-005
Create login UI
```

---

## 19. State Management Specification

Untuk app besar, ini juga berguna.

Misalnya state:

```text
User State
Auth State
Wallet State
Cart State
Checkout State
Notification State
UI State
```

Kemudian tentukan:

```text
Server state → TanStack Query
Global client state → Zustand
Local UI state → React state
URL state → URL params
```

Jangan biarkan setiap developer/agent memilih sendiri.

---

## 20. Design System

Kalau frontend kompleks, dokumentasikan:

```text
Colors
Typography
Spacing
Radius
Shadow
Icons
Buttons
Inputs
Modal
Dropdown
Toast
Table
Card
Navigation
```

Contoh:

```text
Button

Variants:
Primary
Secondary
Danger
Ghost

States:
Default
Hover
Active
Disabled
Loading
```

Ini mencegah UI jadi campur aduk ketika dikerjakan banyak agent.

---

## 21. Configuration & Environment

Misalnya:

```text
.env.local
.env.staging
.env.production
```

Dokumentasikan:

```text
DATABASE_URL
REDIS_URL
S3_BUCKET
STRIPE_SECRET
JWT_SECRET
```

Dan bedakan:

```text
PUBLIC
PRIVATE
SECRET
```

Jangan sampai agent memasukkan secret ke frontend.

---

## 22. Performance Requirements

Aplikasi kompleks perlu target teknis.

Contoh:

```text
Page load < 2.5s
API p95 < 500ms
Search p95 < 300ms
Image size < 500KB
Database query < 100ms
```

Tidak harus angka seperti ini untuk semua aplikasi, tapi **target harus ditentukan**.

---

## 23. Business Rules

Ini berbeda dari technical rules.

Contoh marketplace:

```text
Seller cannot purchase own NFT.

Listing cannot be edited after sale.

Refund allowed only within X conditions.

User cannot withdraw funds below minimum threshold.

Admin can freeze account.
```

Business rules seperti ini sebaiknya punya dokumen sendiri.

---

## 24. Permissions Matrix

Untuk aplikasi dengan banyak role:

| Feature | Guest | User | Seller | Moderator | Admin |
|---|---:|---:|---:|---:|---:|
| Browse | ✓ | ✓ | ✓ | ✓ | ✓ |
| Buy | — | ✓ | ✓ | ✓ | ✓ |
| Sell | — | — | ✓ | ✓ | ✓ |
| Moderate | — | — | — | ✓ | ✓ |
| Admin Panel | — | — | — | — | ✓ |

Ini sangat membantu authorization implementation.

---

## 25. Changelog

Simpan perubahan besar:

```text
CHANGELOG.md
```

Contoh:

```text
v0.1.0
- Initial auth

v0.2.0
- Marketplace

v0.3.0
- Payment
```

---

# Yang Paling Penting untuk AI Agent

Kalau proyekmu memang akan banyak dikerjakan **Claude Code / Codex / agent**, saya justru akan membuat struktur seperti ini:

```text
docs/
│
├── product/
│   ├── PRD.md
│   ├── requirements.md
│   ├── business-rules.md
│   └── user-flows.md
│
├── design/
│   ├── ux-spec.md
│   ├── design-system.md
│   └── components.md
│
├── architecture/
│   ├── system.md
│   ├── frontend.md
│   ├── backend.md
│   ├── infrastructure.md
│   └── decisions/
│
├── data/
│   ├── database.md
│   ├── schema.md
│   └── migrations.md
│
├── api/
│   ├── overview.md
│   ├── endpoints.md
│   └── auth.md
│
├── features/
│   ├── auth.md
│   ├── users.md
│   ├── marketplace.md
│   ├── checkout.md
│   └── ...
│
├── security/
│   ├── security.md
│   ├── roles-permissions.md
│   └── threat-model.md
│
├── testing/
│   ├── strategy.md
│   ├── test-cases.md
│   └── acceptance-criteria.md
│
├── operations/
│   ├── deployment.md
│   ├── monitoring.md
│   ├── backup.md
│   └── disaster-recovery.md
│
└── project/
    ├── roadmap.md
    ├── milestones.md
    ├── backlog.md
    └── changelog.md
```

Lalu di root:

```text
README.md
AGENTS.md
```

---

# Hubungan Antar Dokumen

Yang paling enak dipahami adalah seperti piramida:

```text
                 PRODUCT
                    │
                  PRD
                    │
          REQUIREMENTS / RULES
                    │
               USER FLOWS
                    │
                UX / UI
                    │
              ARCHITECTURE
                    │
            DATABASE / API
                    │
                FEATURES
                    │
                 TASKS
                    │
                  CODE
                    │
                 TESTS
                    │
               PRODUCTION
```

Artinya:

**PRD → apa yang dibangun**

**Requirements → bagaimana behavior produk harus bekerja**

**UX/UI → bagaimana user berinteraksi**

**Architecture → bagaimana software dibangun**

**Database/API → bagaimana komponen bertukar data**

**Feature specs → detail implementasi tiap capability**

**Tasks → pekerjaan konkret**

**Tests → bagaimana kita membuktikan semuanya benar**

---

# Untuk Project yang SANGAT Kompleks

Saya bahkan menyarankan membedakan 4 level.

## Level 1 — Product

```text
PRD
Requirements
Business Rules
User Personas
User Stories
User Flows
```

## Level 2 — System

```text
Architecture
Database
API
Security
Infrastructure
Integrations
```

## Level 3 — Feature

```text
Auth
Marketplace
Payment
Search
Notification
Admin
etc.
```

## Level 4 — Execution

```text
Roadmap
Milestones
Tasks
Testing
Deployment
Monitoring
```

Ini jauh lebih scalable daripada mencoba membuat satu `PRD.md` monster 300 halaman.

---

# Prinsip Penting untuk Vibe Coding + AI Agent

Untuk workflow **vibe coding + AI agent**, saya tidak akan menjadikan `PRD.md` sebagai dokumen paling penting.

Saya akan membuat:

```text
AGENTS.md
        +
docs/
        +
tasks/
```

Karena agent perlu menjawab tiga pertanyaan setiap kali bekerja:

```text
1. WHAT?
   Apa yang harus dibuat?

2. HOW?
   Bagaimana sistem ini seharusnya bekerja?

3. RULES?
   Apa yang tidak boleh dilanggar?
```

PRD menjawab **WHAT**.

Architecture + feature specs menjawab **HOW**.

AGENTS + security + business rules menjawab **RULES**.

Baru kemudian `tasks/` menjawab:

> **"Sekarang gue harus ngerjain yang mana?"**
