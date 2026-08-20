FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TERM=xterm-256color
ENV CI=true

# 1. Install system utilities and development packages
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    wget \
    git \
    jq \
    build-essential \
    python3 \
    python3-pip \
    python3-venv \
    procps \
    psmisc \
    net-tools \
    iproute2 \
    lsof \
    nano \
    vim \
    ripgrep \
    tmux \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# 2. Install Node.js LTS (v20)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# 3. Pre-cache popular agent CLIs (Claude Code, Aider)
RUN npm install -g @anthropic-ai/claude-code || true
RUN pip3 install --no-cache-dir aider-chat || true

WORKDIR /root
CMD ["/bin/sh", "-c", "sleep infinity"]
