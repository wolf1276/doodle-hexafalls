This is a [Next.js](https://nextjs.org) project bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

## PayGig Escrow Program

The on-chain Solana Anchor program that holds and releases milestone payments lives in `programs/escrow`. It is **production-ready**: architecture finalized, full instruction set and PDA design implemented, SPL Token integration complete, 106-test suite passing, and a full internal security audit completed with no open findings.

- [ARCHITECTURE.md](./ARCHITECTURE.md) — account model, instruction flow, state machine, and PDA architecture.
- [SECURITY.md](./SECURITY.md) — completed audit: threat model and every enforced invariant.
- [TESTING.md](./TESTING.md) — breakdown of the 106-test, 11-module test suite.
- [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) — completion status across all on-chain programs.
- [CHANGELOG.md](./CHANGELOG.md) — release history.

Run the program's test suite with:

```bash
cargo test -p escrow
```

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.
