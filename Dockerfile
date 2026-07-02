FROM mcr.microsoft.com/playwright:v1.44.0-jammy

WORKDIR /workspace

RUN npm install -g pnpm bun yarn

RUN apt-get update && apt-get install -y \
    git \
    curl \
    jq \
    vim \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash evaluser
RUN chown -R evaluser:evaluser /workspace

USER evaluser

CMD ["/bin/bash"]
