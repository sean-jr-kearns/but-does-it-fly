# Use official Rust image
FROM rust:1.98.0

# Args
ARG USERNAME
ARG UID
ARG GID

# Rename the default 'ubuntu' user if it exists
RUN if id "ubuntu" >/dev/null 2>&1; then \
        # Rename username and group, and update home directory path
        usermod -l ${USERNAME} -d /home/${USERNAME} -m ubuntu && \
        groupmod -n ${USERNAME} ubuntu; \
    fi

# Install dependencies
RUN apt-get update && apt-get install -y \
    sudo \
    locales \
    pkg-config \
    libssl-dev \
    lldb \
    curl \
    git \
    build-essential \
    ca-certificates \
    python3 \
    python3-pip \
    nodejs \
    npm \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Ensure locale
RUN locale-gen en_US.UTF-8
ENV LANG=en_US.UTF-8

# Set user directory permissions
RUN mkdir -p /home/${USERNAME}/ && \
    chown -R ${UID}:${GID} /home/${USERNAME}/ && \
    chmod -R a+rwx /home/${USERNAME}/ && \
    chown -R ${UID}:${GID} /usr/local/cargo

# Install some cargo tools
RUN cargo install cargo-watch cargo-edit cargo-tarpaulin

# Working directory
WORKDIR /workspace

# Set user
USER ${USERNAME}

CMD ["sleep", "infinity"]
