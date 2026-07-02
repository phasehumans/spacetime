# Use the official Playwright image which comes with Node.js and all browser dependencies pre-installed
FROM mcr.microsoft.com/playwright:v1.44.0-jammy

# Set up a working directory for the evaluation
WORKDIR /workspace

# Install common package managers (npm is already included)
RUN npm install -g pnpm bun yarn

# Install useful utilities for debugging and interacting
RUN apt-get update && apt-get install -y \
    git \
    curl \
    jq \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run tests safely
RUN useradd -m -s /bin/bash evaluser
RUN chown -R evaluser:evaluser /workspace

# Switch to the evaluser
USER evaluser

# By default, start an interactive shell
CMD ["/bin/bash"]
