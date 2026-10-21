import { defineConfig } from "astro/config";

// Static output: the leaderboard is data baked at build time, so there is no
// server to run and no API to attack. Deployed from the repo root via vercel.json.
export default defineConfig({
  site: "https://spacetime.trydecember.com",
  output: "static",
  build: {
    format: "directory",
  },
});
