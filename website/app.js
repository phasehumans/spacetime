const agentsData = [
  {
    name: "claude code",
    model: "claude-3-7-sonnet",
    passRate: "90.0%",
    passed: 18,
    total: 20,
    avgLatency: "2.4s"
  },
  {
    name: "codex cli",
    model: "o3-mini",
    passRate: "85.0%",
    passed: 17,
    total: 20,
    avgLatency: "3.1s"
  },
  {
    name: "antigravity",
    model: "gemini-2.5-pro",
    passRate: "80.0%",
    passed: 16,
    total: 20,
    avgLatency: "1.9s"
  },
  {
    name: "cursor cli",
    model: "claude-3-5-sonnet",
    passRate: "80.0%",
    passed: 16,
    total: 20,
    avgLatency: "2.8s"
  },
  {
    name: "openhands",
    model: "deepseek-r1",
    passRate: "75.0%",
    passed: 15,
    total: 20,
    avgLatency: "4.8s"
  },
  {
    name: "aider",
    model: "claude-3-5-sonnet",
    passRate: "70.0%",
    passed: 14,
    total: 20,
    avgLatency: "3.6s"
  },
  {
    name: "december",
    model: "claude-3-7-sonnet",
    passRate: "70.0%",
    passed: 14,
    total: 20,
    avgLatency: "2.1s"
  },
  {
    name: "devin",
    model: "gpt-4o",
    passRate: "65.0%",
    passed: 13,
    total: 20,
    avgLatency: "3.5s"
  },
  {
    name: "swe-agent",
    model: "qwen-2.5-72b",
    passRate: "65.0%",
    passed: 13,
    total: 20,
    avgLatency: "5.2s"
  },
  {
    name: "pi",
    model: "deepseek-v3",
    passRate: "60.0%",
    passed: 12,
    total: 20,
    avgLatency: "3.8s"
  }
];

function renderLeaderboard() {
  const container = document.getElementById("leaderboard-body");
  if (!container) return;
  container.innerHTML = agentsData.map((agent, index) => {
    return `
      <div class="lb-row">
        <span class="rank-val">${index + 1}</span>
        <div class="model-val">
          <span>${agent.name}</span>
          <span class="model-tag">${agent.model}</span>
        </div>
        <span class="col-num pass-val">${agent.passRate}</span>
        <span class="col-num col-hide-mobile">${agent.passed}/${agent.total}</span>
        <span class="col-num col-hide-mobile" style="color: var(--text-muted);">${agent.avgLatency}</span>
      </div>
    `;
  }).join("");
}

let copyTimeout = null;

function copyInstallCmd() {
  const cmd = "curl -fsSL https://spacetime.trydecember.com | bash";
  navigator.clipboard.writeText(cmd).then(() => {
    const btn = document.getElementById("btn-copy-icon");
    
    if (btn) {
      btn.classList.add("copied");
      btn.innerHTML = `
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      `;
    }

    clearTimeout(copyTimeout);
    copyTimeout = setTimeout(() => {
      if (btn) {
        btn.classList.remove("copied");
        btn.innerHTML = `
          <svg id="icon-copy" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        `;
      }
    }, 2000);
  });
}

document.addEventListener("DOMContentLoaded", renderLeaderboard);
