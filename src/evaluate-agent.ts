import { exec, spawn } from "child_process";
import * as path from "path";
import * as fs from "fs/promises";
import { promisify } from "util";

const execAsync = promisify(exec);

async function setupAgentEnvironment(repoUrl: string, commitSha: string, repoName: string) {
    const workspaceDir = path.resolve(__dirname, "..", "eval_workspace");
    const repoDir = path.join(workspaceDir, repoName);

    await fs.mkdir(workspaceDir, { recursive: true });

    const dirExists = await fs.stat(repoDir).then(() => true).catch(() => false);
    if (dirExists) {
        await fs.rm(repoDir, { recursive: true, force: true });
    }

    console.log(`Cloning ${repoUrl} for the agent...`);
    await execAsync(`git clone ${repoUrl} ${repoName}`, { cwd: workspaceDir });
    await execAsync(`git checkout ${commitSha}`, { cwd: repoDir });
    await execAsync(`chmod -R 777 ${repoDir}`);
    
    return repoDir;
}

export async function evaluateAgent(issue: any) {
    console.log(`\n======================================================`);
    console.log(`🤖 Starting Agent Harness for: ${issue.repo}#${issue.pr_number}`);
    console.log(`======================================================`);
    
    const repoName = issue.repo.split('/')[1];
    const repoDir = await setupAgentEnvironment(issue.url, issue.base_sha, repoName);

    console.log(`\n[Agent Prompt Initialized]`);
    console.log(`System: You are an expert AI software engineer. You have access to a terminal in a sandboxed Docker container where this repository is mounted at /workspace/repo.`);
    console.log(`Task: Fix the following issue: "${issue.title}"`);
    console.log(`Instructions: Run the tests, explore the codebase, edit the files, and verify the tests pass. When you are done, exit.`);
    
    // In a real scenario, this is where we would invoke the LLM agent and provide it 
    // with a tool loop to execute commands in the docker container.
    console.log(`\n⏳ Waiting for Agent to complete its work... (Simulated)`);
    
    // For demonstration, we pause here. In a real system, we'd wait for the LLM's final completion signal.
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log(`\nAgent signaled completion. Verifying fix...`);
    
    const testCommand = "cd /workspace/repo && if [ -f pnpm-lock.yaml ]; then pnpm install; elif [ -f yarn.lock ]; then yarn install; else npm install; fi && npx playwright test";
    const containerCmd = [
        "docker", "run", "--rm",
        "-v", `${repoDir}:/workspace/repo`,
        "benchmark-eval-env",
        "bash", "-c", testCommand
    ];

    return new Promise((resolve) => {
        const dockerRun = spawn(containerCmd[0], containerCmd.slice(1), { stdio: 'inherit' });

        dockerRun.on('close', (code) => {
            if (code === 0) {
                console.log(`\n✅ RESULT: Pass@1 = TRUE (Agent successfully fixed ${issue.repo})`);
                resolve(true);
            } else {
                console.error(`\n❌ RESULT: Pass@1 = FALSE (Agent failed to fix ${issue.repo})`);
                resolve(false);
            }
        });
    });
}

// Example usage if run directly
if (require.main === module) {
    // Mock issue data that would normally come from dataset.json
    const mockIssue = {
        repo: "opencollection-dev/opencollection",
        pr_number: 54,
        base_sha: "HEAD~1", // typically this is the base commit hash
        title: "feat(oc-docs): endpoint search palette",
        url: "https://github.com/opencollection-dev/opencollection"
    };
    evaluateAgent(mockIssue);
}
