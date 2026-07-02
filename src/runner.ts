import { exec, spawn } from "child_process";
import * as path from "path";
import * as fs from "fs/promises";
import { promisify } from "util";

const execAsync = promisify(exec);

export async function runEvaluation(repoUrl: string, commitSha: string): Promise<boolean> {
    console.log(`Preparing evaluation environment for ${repoUrl} at ${commitSha}...`);

    try {
        console.log("Building Docker image...");
        await execAsync('docker build -t benchmark-eval-env .', { cwd: path.resolve(__dirname, "..") });
        console.log("Docker image built successfully!");

        const repoName = repoUrl.split('/').pop()?.replace('.git', '') || 'temp_repo';
        const workspaceDir = path.resolve(__dirname, "..", "eval_workspace");
        const repoDir = path.join(workspaceDir, repoName);

        await fs.mkdir(workspaceDir, { recursive: true });

        const dirExists = await fs.stat(repoDir).then(() => true).catch(() => false);
        if (dirExists) {
            console.log(`Cleaning up existing repo directory at ${repoDir}...`);
            await fs.rm(repoDir, { recursive: true, force: true });
        }

        console.log(`Cloning repository into ${repoDir}...`);
        await execAsync(`git clone ${repoUrl} ${repoName}`, { 
            cwd: workspaceDir,
            env: { ...process.env, GIT_TERMINAL_PROMPT: "0" }
        });

        console.log(`Checking out commit ${commitSha}...`);
        await execAsync(`git checkout ${commitSha}`, { 
            cwd: repoDir,
            env: { ...process.env, GIT_TERMINAL_PROMPT: "0" }
        });

        console.log("Fixing permissions for Docker mount...");
        await execAsync(`chmod -R 777 ${repoDir}`);

        console.log("Running evaluation in Docker container...");
        
        const testCommand = "cd /workspace/repo && if [ -f pnpm-lock.yaml ]; then pnpm install; elif [ -f yarn.lock ]; then yarn install; else npm install; fi && npx playwright test";
        
        const containerCmd = [
            "docker", "run", "--rm",
            "-v", `${repoDir}:/workspace/repo`,
            "benchmark-eval-env",
            "bash", "-c", testCommand
        ];

        return new Promise((resolve, reject) => {
            const dockerRun = spawn(containerCmd[0], containerCmd.slice(1), { stdio: 'inherit' });

            dockerRun.on('close', (code) => {
                if (code === 0) {
                    console.log("Evaluation completed successfully! Tests passed.");
                    resolve(true);
                } else {
                    console.error(`Evaluation failed. Tests exited with code ${code}.`);
                    resolve(false);
                }
            });
        });

    } catch (error) {
        console.error("An error occurred during the evaluation process:", error);
    }
}
