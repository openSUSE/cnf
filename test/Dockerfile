# Tumbleweed based CI image for cnf integration testing
FROM registry.opensuse.org/opensuse/tumbleweed

ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

RUN zypper refresh
# minimize number of packages installed
RUN zypper --non-interactive addlock busybox dbus diffutils pam-config perl-base pkgconf-m4 systemd groff man
RUN zypper --non-interactive install --no-recommends zsh fish

WORKDIR /src
