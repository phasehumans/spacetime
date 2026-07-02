import { spawn } from "child_process";
import * as path from "path";

async function runEvaluation(repoUrl: string, commitSha: string) {
    console.log(`Preparing evaluation environment...`);
    
    // Step 1: Build the Docker image
    console.log("Building Docker image (this may take a few minutes the first time)...");
    
    const dockerBuild = spawn("docker", ["build", "-t", "benchmark-eval-env", "."], { 
        cwd: path.resolve(__dirname, ".."),
        stdio: "inherit" 
    });

    dockerBuild.on("close", (code) => {
        if (code !== 0) {
            console.error("❌ Docker build failed.");
            return;
        }

        console.log("✅ Docker image built successfully!");
        console.log(`\nNext steps for evaluating ${repoUrl} at ${commitSha}:`);
        console.log(`1. Clone the repository to a temporary workspace.`);
        console.log(`2. Start the container, mounting the workspace:`);
        console.log(`   docker run --rm -v $(pwd)/temp_repo:/workspace/repo benchmark-eval-env`);
        console.log(`3. Inside the container, run the package manager (npm/yarn/pnpm) and Playwright tests.`);
        console.log(`4. Parse the exit code to determine Pass@1 status.`);
    });
}

// Example invocation
runEvaluation("https://github.com/opencollection-dev/opencollection", "HEAD");
