const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

async function searchMergedPRs() {
    // Search for merged PRs in JS/TS that likely contain Playwright or Cypress tests
    // Filtering out common bots and dependency labels
    const query = 'is:pr is:merged language:typescript "playwright" -author:app/dependabot -author:app/renovate -author:dependabot -author:renovate -label:dependencies';
    const url = `https://api.github.com/search/issues?q=${encodeURIComponent(query)}&sort=updated&order=desc&per_page=10`;

    const headers: Record<string, string> = {
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'Benchmark-Collector'
    };

    if (GITHUB_TOKEN) {
        headers['Authorization'] = `Bearer ${GITHUB_TOKEN}`;
    } else {
        console.warn("⚠️ No GITHUB_TOKEN provided. You may hit rate limits quickly.");
    }

    console.log(`Searching GitHub for: ${query}\n`);
    try {
        const response = await fetch(url, { headers });
        if (!response.ok) {
            throw new Error(`GitHub API returned ${response.status}: ${await response.text()}`);
        }
        
        const data = await response.json();
        console.log(`Total matching PRs found (first 10 shown): ${data.total_count}\n`);
        
        for (const item of data.items) {
            // Extract repo name from repository_url: https://api.github.com/repos/OWNER/REPO
            const repoParts = item.repository_url.split('/');
            const repoName = `${repoParts[repoParts.length - 2]}/${repoParts[repoParts.length - 1]}`;
            
            console.log(`- [${repoName}] ${item.title}`);
            console.log(`  URL: ${item.html_url}`);
            console.log('');
        }
    } catch (error) {
        console.error("Error searching GitHub:", error);
    }
}

searchMergedPRs();
