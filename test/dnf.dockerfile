# Tumbleweed based CI image for cnf integration testing with dnf4
FROM registry.opensuse.org/opensuse/tumbleweed

# This is the same as dnf5.dockerfile
ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

# The following is exactly the same as for dnf5, except it uses 'dnf' instead of 'dnf5'
RUN zypper refresh
# As with dnf5, dnf4 fails to work without the libcurl4 package
RUN zypper --non-interactive install --no-recommends dnf libcurl4 rpm-repos-openSUSE-Tumbleweed
RUN dnf --assumeyes remove zypper libzypp
RUN rm -rf /etc/zypp /var/cache/zypp/ /var/lib/zypp

RUN echo excludepkgs=busybox dbus diffutils pam-config perl-base systemd groff man >> /etc/dnf/dnf.conf
RUN dnf --setopt install_weak_deps=False --assumeyes install zsh fish libsolv1
RUN curl -O https://download.opensuse.org/repositories/GNOME:/Next/openSUSE_Factory/GNOME:Next.repo --output-dir /etc/dnf/repos.d/
RUN dnf makecache

WORKDIR /src
