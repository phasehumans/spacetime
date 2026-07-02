import { getCandidatePRs, getPRDetails } from "./collect.js";
import { runEvaluation } from "./runner.js";
import * as fs from "fs/promises";
import * as path from "path";

async function buildDataset() {
    console.log("Starting dataset build pipeline...");
    
    // We start by fetching a small batch of candidate PRs
    const prs = await getCandidatePRs(5); 
    const dataset = [];

    for (const pr of prs) {
        const repoParts = pr.repository_url.split('/');
        const repoOwner = repoParts[repoParts.length - 2];
        const repoName = repoParts[repoParts.length - 1];
        const fullName = `${repoOwner}/${repoName}`;
        
        console.log(`\n\n======================================================`);
        console.log(`Processing PR: ${fullName}#${pr.number}`);
        console.log(`======================================================`);
        
        try {
            const details = await getPRDetails(fullName, pr.number);
            const baseSha = details.base.sha;
            const headSha = details.head.sha;
            const repoUrl = details.head.repo?.clone_url || details.base.repo?.clone_url;

            if (!repoUrl) {
                console.log(`❌ Skipping: Could not determine clone URL.`);
                continue;
            }

            console.log(`Base SHA: ${baseSha} (Before Fix)`);
            console.log(`Head SHA: ${headSha} (After Fix)`);

            // Step 1: Test the HEAD commit (should pass because it has the fix)
            console.log(`\n--- Step 1: Evaluating HEAD commit (expecting PASS) ---`);
            const headPassed = await runEvaluation(repoUrl, headSha);
            
            if (!headPassed) {
                console.log(`❌ Skipping ${fullName}#${pr.number}: Tests failed on the HEAD commit (environment/build issues).`);
                continue;
            }

            // Step 2: Test the BASE commit (should fail because it's missing the fix)
            console.log(`\n--- Step 2: Evaluating BASE commit (expecting FAIL) ---`);
            const basePassed = await runEvaluation(repoUrl, baseSha);

            if (basePassed) {
                console.log(`❌ Skipping ${fullName}#${pr.number}: Tests passed on BASE commit (couldn't reproduce the bug).`);
                continue;
            }

            // Step 3: Golden PR Found!
            console.log(`\n✅ SUCCESS! Found a golden reproducible PR: ${fullName}#${pr.number}`);
            dataset.push({
                repo: fullName,
                pr_number: pr.number,
                base_sha: baseSha,
                head_sha: headSha,
                title: pr.title,
                url: pr.html_url
            });

            // Save continuously so we don't lose progress
            const datasetPath = path.resolve(__dirname, "..", "dataset.json");
            await fs.writeFile(datasetPath, JSON.stringify(dataset, null, 2));
            console.log(`💾 Saved to dataset.json`);

        } catch (err) {
            console.error(`Error processing PR ${fullName}#${pr.number}:`, err);
        }
    }

    console.log(`\n🎉 Dataset build complete. Found ${dataset.length} golden PRs.`);
}

buildDataset();
