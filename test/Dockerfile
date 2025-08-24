# Tumbleweed based CI image for cnf integration testing
FROM registry.opensuse.org/opensuse/tumbleweed

ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

RUN zypper refresh
# minimize number of packages installed
RUN zypper --non-interactive addlock busybox dbus diffutils pam-config perl-base pkgconf-m4 systemd groff man
RUN zypper --non-interactive install --no-recommends zsh fish libsolv1
RUN zypper --non-interactive addrepo https://download.opensuse.org/repositories/GNOME:/Next/openSUSE_Factory/GNOME:Next.repo
RUN zypper --gpg-auto-import-keys refresh GNOME_Next

WORKDIR /src
