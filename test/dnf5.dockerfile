# Tumbleweed based CI image for cnf integration testing with dnf5
FROM registry.opensuse.org/opensuse/tumbleweed

ADD passwd /etc/passwd
ADD group /etc/group
ADD --chown=65532:65532 nonroot /home/nonroot/whoami

RUN zypper refresh
# dnf5 completely fails to work without the libcurl4 package, yet it isn't transitvely required, recommended, or suggested
RUN zypper --non-interactive install --no-recommends dnf5 libcurl4 rpm-repos-openSUSE-Tumbleweed
RUN dnf5 --assumeyes remove zypper libzypp
# Ensure zypper's cache and settings aren't installed
RUN rm -rf /etc/zypp /var/cache/zypp/ /var/lib/zypp

# The rest here is the equivalent of Zypper-Dockerfile, but changed to use dnf5
# Equivelent to zypper addlock
RUN echo excludepkgs=busybox dbus diffutils pam-config perl-base systemd groff man >> /etc/dnf/dnf.conf
# --setopt install_weak_deps=False is equivalent to to zypper --no-recommends
RUN dnf5 --setopt install_weak_deps=False --assumeyes install zsh fish libsolv1
# Equivalent of zypper addrepo
RUN curl -O https://download.opensuse.org/repositories/GNOME:/Next/openSUSE_Factory/GNOME:Next.repo --output-dir /etc/dnf/repos.d/
# Equivalent of zypper refresh (note that the default dnf5 settings don't check GPG-keys)
RUN dnf5 makecache

WORKDIR /src
