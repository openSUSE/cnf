# Tumbleweed based CI image for cnf integration testing with zypper
FROM registry.opensuse.org/opensuse/tumbleweed

ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

# download.opensuse.org redirects to a mirror, and mirrors sometimes lag behind
# the published repomd.xml and answer 404 for the metadata files it lists, which
# breaks the refresh. Use the origin server, it is always self-consistent.
RUN sed -i 's|//download\.opensuse\.org|//downloadcontent.opensuse.org|' /etc/zypp/repos.d/*.repo
RUN zypper refresh
# minimize number of packages installed
RUN zypper --non-interactive addlock busybox dbus diffutils pam-config perl-base pkgconf-m4 systemd groff man
RUN zypper --non-interactive install --no-recommends zsh fish libsolv1
RUN zypper --non-interactive addrepo https://downloadcontent.opensuse.org/repositories/GNOME:/Next/openSUSE_Factory/GNOME:Next.repo
RUN sed -i 's|//download\.opensuse\.org|//downloadcontent.opensuse.org|' /etc/zypp/repos.d/*.repo
RUN zypper --gpg-auto-import-keys refresh GNOME_Next

WORKDIR /src
