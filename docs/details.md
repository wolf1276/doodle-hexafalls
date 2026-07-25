PayGig
The Payment Rail for Freelancers — Instant, Fair, Portable. Built on Solana.
1. The Problem
Freelance platforms like Upwork and Fiverr charge 10–20% in fees and hold payouts for 7–14 days. Clients pay with no real protection if work is bad; freelancers deliver work and then risk being ghosted with their money stuck indefinitely. Reputation is also trapped — leave the platform and your track record is gone.

2. The Solution
PayGig is a freelance marketplace that looks and works like any gig platform — post a job, hire, deliver, get paid — but underneath:

Payments are locked in milestone-based on-chain escrow, not held by a company
Approved work is paid instantly (seconds, not weeks)
If a client goes silent after delivery, the freelancer is protected by an automatic partial release, not left waiting indefinitely
Disagreements go to a neutral jury, not a platform support ticket
Reputation is a portable, on-chain asset the freelancer owns — not locked to one platform
Platform fee: 0.5%, instead of 10–20%
3. Niche / Wedge
Launching focused on AI data-labeling and prompt-engineering gig work — a fast-growing, high-volume category with short, easily verified tasks and no dominant trusted payment rail yet. The mechanics generalize to any gig category; this is a deliberate entry point, not a ceiling.

4. System Architecture
4.1 Layers
Layer	Component	Role
Client	Privy	Email/social login, embedded wallet, gasless transactions
Client	Next.js app	Listings, hire flow, dashboard, wallet adapter
On-chain	Escrow program	Milestone lock/release, submission timer, partial + full timeout release, dispute trigger
On-chain	Dispute/jury program	Evidence submission, juror voting, resolution
On-chain	Reputation program	Points accumulation, badge minting at 100 pts
External	Shadow Drive	Delivery file storage
External	MoonPay	USDC-to-fiat off-ramp
4.2 Why Privy
Users sign up with just an email or social login. Privy creates an embedded wallet behind the scenes — no seed phrase, no browser extension, no gas token needed (transactions are sponsored). This removes the single biggest reason non-crypto users bounce off a Web3 product, and is the backbone of PayGig's "feels like a normal app" UX.

4.3 Program relationships
Escrow → Reputation: on approve_milestone, a CPI calls submit_rating, adding points automatically — can't be skipped or faked client-side.
Escrow → Dispute: on raise_dispute, control passes to the dispute program; remaining escrow funds freeze until resolve_case.
Reputation → Badge mint: submit_rating checks if total_points crossed a 100-point multiple; if so, calls mint_reputation_badge automatically.
Reputation → Public API: an off-chain indexer reads on-chain profile + badge data (via DAS API) and exposes GET /reputation/:wallet for any external platform to query.
5. Escrow Program — Detailed Logic
5.1 Accounts
Gig — client, freelancer, milestone list, amounts, status
EscrowVault (PDA) — holds locked USDC per gig
5.2 Instructions
Instruction	Trigger	Effect
create_gig	Client posts a gig	Off-chain listing + on-chain Gig account created
fund_milestone	Client pays for a milestone	Locks 100% of milestone amount in EscrowVault
submit_delivery	Freelancer delivers work	Records submitted_at timestamp, starts the response timer
approve_milestone	Client approves + rates (4.0–5.0)	Releases 100% to freelancer; CPI to submit_rating
partial_timeout_release	72h pass with no client action	Auto-releases 20% of the milestone to the freelancer; remaining 80% stays locked; no reputation points awarded
full_timeout_release	7 days pass with no client action	Auto-releases the remaining 80% to the freelancer; no reputation points awarded
raise_dispute	Either party disagrees, before full release	Freezes remaining balance; hands off to dispute program
5.3 Why the partial-release safety net matters
This directly fixes freelancers' most common real complaint: doing the work and then being ghosted with funds stuck indefinitely. The client still controls 80% of the outcome (so they're not stripped of leverage), but the freelancer is never left with zero recourse. Reputation points are only ever awarded on genuine client approval — a partial or full timeout release means "the client didn't respond," not "the client was happy," so it correctly doesn't affect the score.

6. Dispute / Jury Program — Detailed Logic
6.1 Accounts
DisputeCase — gig reference, evidence hashes, vote tally, status
JurorPool — staked, eligible voters (hackathon scope: 3 fixed juror wallets)
6.2 Instructions
Instruction	Trigger	Effect
open_case	Auto-created from raise_dispute	New DisputeCase initialized, funds already frozen
submit_vote	Juror reviews evidence	Records a release-to-freelancer / refund-to-client vote
resolve_case	Voting window closes	Majority vote determines final release of remaining funds; majority-voting jurors earn a small fee, minority-voting jurors may lose a portion of stake
6.3 What it replaces
Traditional platforms resolve disputes via opaque support tickets, with the platform itself as an interested party. This program makes resolution transparent, evidence-based, incentive-aligned, and auditable on-chain — no human at PayGig ever touches the disputed funds directly.

7. Reputation Program — Detailed Logic
7.1 Accounts
FreelancerProfile (PDA, seeded by wallet) — wallet, total_points, gigs_completed, badges_minted
7.2 Instructions
Instruction	Trigger	Effect
submit_rating	CPI from approve_milestone only	Adds points: 4.0 → 5, 4.5 → 10, 5.0 → 15; below 4.0 → 0. Purely additive, no negative points, ever.
mint_reputation_badge	Auto-fires when total_points crosses a 100 multiple	Mints a compressed NFT (Metaplex Bubblegum) badge to the freelancer's wallet; increments badges_minted
7.3 Why points-to-badge instead of an NFT per gig
Cheaper, simpler to demo, and reads as a familiar loyalty/achievement mechanic ("earn 100 points, unlock a verified badge") rather than a crypto-native concept — instantly understandable to non-crypto judges. Because the data is on-chain and exposed via an open API, any platform can verify a freelancer's real track record without trusting or even integrating with PayGig directly.

8. End-to-End Workflow
8.1 Happy path
User signs up via Privy (email) → embedded wallet created silently
Client posts a gig, funds the first milestone → USDC locked in escrow vault
Freelancer delivers work → file uploaded to Shadow Drive, submit_delivery starts the timer
Client approves + rates within the window → approve_milestone releases funds instantly, CPI adds points
If points cross 100 → badge cNFT mints automatically into the freelancer's wallet
Freelancer taps "Cash out" → MoonPay off-ramp sends USDC to their bank account
Any external platform can query the freelancer's public reputation via the open API
8.2 Ghosting-protection path
Freelancer delivers, client doesn't respond within 72 hours
partial_timeout_release auto-fires → 20% released to freelancer, 80% remains locked
Client later approves → remaining 80% releases, rating/points apply normally or client disputes → remaining 80% goes to jury program or client stays silent past 7 days total → full_timeout_release releases the remaining 80%, no points awarded
8.3 Dispute path
At approval or during the timeout window, either party raises a dispute instead
raise_dispute opens a DisputeCase, freezes remaining funds
Jurors review evidence and vote within a window
resolve_case releases funds per majority ruling; no points awarded on a disputed outcome
9. Demo Script (3 minutes)
Problem (20s): high fees, slow payouts, no ghosting protection, no portable reputation on Upwork/Fiverr
Solution (20s): instant milestone payouts, a ghosting safety net, fair disputes, portable reputation
Live demo (100s):
Sign up with just an email via Privy — no wallet extension, no seed phrase
Hire a freelancer, fund a milestone
Freelancer delivers, client approves with 5 stars → USDC lands in wallet in ~2 seconds, points update live
Freelancer pre-seeded at 90 points → this rating crosses 100 → badge cNFT mints on screen
Second scenario: show a pre-set delivery past the timeout window → 20% auto-releases live to demonstrate the ghosting protection
Tap "Cash out" → funds land via MoonPay
Close (20s): fee/payout comparison table vs. Upwork/Fiverr, 0.5% take-rate business model, next steps
10. Tech Stack
On-chain: Anchor (Rust) — three programs: escrow, dispute/jury, reputation
Payments: USDC via SPL transfers
Onboarding: Privy — embedded wallets, social/email login, sponsored transactions
Reputation: Metaplex Bubblegum compressed NFTs + open query API (DAS-indexed)
Storage: Shadow Drive (Solana-native)
Off-ramp: MoonPay
Frontend: Next.js + @solana/wallet-adapter
Backend: Postgres/Supabase for listings, users, search
11. 36-Hour Build Plan
Phase	Focus
Hours 0–8	Escrow program: create_gig, fund_milestone, submit_delivery, approve_milestone
Hours 8–16	Marketplace UI + Privy integration + wallet adapter; wire hire → fund → deliver → approve flow
Hours 16–22	Reputation program: submit_rating CPI, points logic, mint_reputation_badge; wire into approval flow
Hours 22–28	Partial/full timeout release logic (partial_timeout_release, full_timeout_release)
Hours 28–32	Simplified 3-juror dispute program (open_case, submit_vote, resolve_case) — only if on schedule
Hours 32–34	Off-ramp (MoonPay) integration, polish UI on the 3 screens judges will see
Hours 34–36	Seed demo data, rehearse pitch 3x, record backup demo video
Cut-scope rule: if escrow + reputation aren't fully stable by hour 20, drop the dispute program entirely and lean on the partial-release safety net as your second demo moment instead.

12. Why This Wins on the Judging Criteria
Criterion	How PayGig delivers
Functionality	Full working loop — escrow, timeout protection, reputation, and (if time allows) dispute resolution — demoable live end to end
Potential impact	Freelance economy is a massive, real market; ghosting and slow payouts are universal, relatable pain points
Novelty	Ghosting-protection partial release + points-to-badge reputation + jury dispute resolution are genuinely differentiated mechanisms, not generic "escrow on-chain"
UX	Privy makes onboarding indistinguishable from signing up for any normal app; no wallets, no seed phrases, no gas
Open-source/composability	Public reputation API is a reusable primitive any other platform can query
Business plan	0.5% take rate on GMV, credible niche entry (AI labeling gigs), clear expansion path to other gig categories
13. Validation Plan
Before the pitch: post in 1–2 AI-labeling/freelance Discord or subreddit communities, get 5–10 people to try the flow on devnet with test funds, and collect direct quotes. Real user feedback ("8 freelancers tested this, 100% said they'd switch") is worth more on stage than any additional feature