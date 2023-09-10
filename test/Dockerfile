# Tumbleweed based CI image for cnf-rs integration testing
FROM registry.opensuse.org/opensuse/tumbleweed

ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

# Needed for /var/cache/zypp/solv to appear
RUN zypper refresh --build-only

WORKDIR /src
