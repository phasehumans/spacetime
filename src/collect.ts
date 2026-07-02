const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

export async function getCandidatePRs(limit = 10): Promise<any[]> {
    const query = 'is:pr is:merged language:typescript "playwright" -author:app/dependabot -author:app/renovate -author:dependabot -author:renovate -label:dependencies';
    const url = `https://api.github.com/search/issues?q=${encodeURIComponent(query)}&sort=updated&order=desc&per_page=${limit}`;

    const headers: Record<string, string> = {
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'Benchmark-Collector'
    };

    if (GITHUB_TOKEN) {
        headers['Authorization'] = `Bearer ${GITHUB_TOKEN}`;
    }

    console.log(`Searching GitHub for potential PRs...`);
    try {
        const response = await fetch(url, { headers });
        if (!response.ok) {
            throw new Error(`GitHub API returned ${response.status}: ${await response.text()}`);
        }
        
        const data = await response.json();
        console.log(`Total matching PRs found: ${data.total_count}`);
        
        return data.items;
    } catch (error) {
        console.error("Error searching GitHub:", error);
        return [];
    }
}

export async function getPRDetails(repoName: string, prNumber: number): Promise<any> {
    const url = `https://api.github.com/repos/${repoName}/pulls/${prNumber}`;
    const headers: Record<string, string> = {
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'Benchmark-Collector'
    };
    if (GITHUB_TOKEN) headers['Authorization'] = `Bearer ${GITHUB_TOKEN}`;

    const response = await fetch(url, { headers });
    if (!response.ok) {
        throw new Error(`Failed to fetch PR details: ${response.status}`);
    }
    return response.json();
}
