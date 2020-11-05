FROM rust:buster

# Options for setup script
ARG DEBIAN_FRONTEND=noninteractive
ARG CARGO_TARGET_DIR=/tmp
ENV CARGO_TARGET_DIR=${CARGO_TARGET_DIR}

# Install system packages
RUN curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | bash && \
    apt-get update && \
    apt-get -y install --no-install-recommends llvm-dev libclang-dev clang cmake pkg-config libsmbclient-dev smbclient git-lfs

# Add Rust components
RUN rustup component add rustfmt clippy rust-src rust-analysis rls && \
    chmod 0777 /usr/local/cargo -R

# Create a non-root user to use if preferred - see https://aka.ms/vscode-remote/containers/non-root-user.
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
ARG DEBIAN_FRONTEND=noninteractive
RUN groupadd --gid $USER_GID $USERNAME && \
    useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME && \
    # [Optional] Add sudo support for the non-root user
    apt-get install -y sudo && \
    echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME
