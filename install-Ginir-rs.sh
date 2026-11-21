#!/usr/bin/env bash
# install-Ginir-rs.sh - install everything the iNiR / Ginir-rs shell needs on Gentoo.
#
# One self-contained dependency installer for a Gentoo (OpenRC or systemd) box:
#   * ::gentoo packages for the compositor, Qt/KDE runtime, portals, audio,
#     screenshots, OCR, fonts and desktop tools
#   * ::guru packages (niri, awww, cliphist, wtype, xwayland-satellite, ...)
#   * the few things without an ebuild (gum, Material Symbols, JetBrains Mono NF)
#
# Usage:
#   sudo ./install-Ginir-rs.sh [options]
#
# Options:
#   -y, --yes              do not ask for confirmation
#   -n, --dry-run          print the plan, change nothing
#       --no-audio         skip the audio group
#       --no-toolkit       skip the toolkit group
#       --no-screencapture skip screenshot/OCR group
#       --no-fonts         skip fonts and themes
#       --no-guru          do not enable/use the ::guru overlay
#       --with-quickshell  also install gui-apps/quickshell (source build)
#       --with-webengine   also install Qt WebEngine + layer-shell-qt
#       --with-sddm        also install SDDM
#       --with-rust        install the Rust toolchain for Inir-rs development
#       --with-extra-themes install Darkly, Capitaine cursors and WhiteSur icons
#       --with-optional    install Cloudflare WARP, Ollama, whisper.cpp, Flatpak
#       --enable-services  enable polkit, power-profiles-daemon and bluetooth
#       --add-groups       add the invoking user to input/video/render/audio
#   -h, --help             show this help
#
# The script is idempotent: running it again only fills in what is missing.
# It uses `emerge --noreplace --oneshot`, so nothing is added to @world.

set -Eeuo pipefail

SCRIPT_NAME=${0##*/}
ORIG_ARGS=("$@")
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

if [[ -t 1 ]]; then
  C_RESET=$'\033[0m'
  C_INFO=$'\033[36m'
  C_OK=$'\033[32m'
  C_WARN=$'\033[33m'
  C_ERR=$'\033[31m'
else
  C_RESET='' C_INFO='' C_OK='' C_WARN='' C_ERR=''
fi

info() { printf '%s[*]%s %s\n' "$C_INFO" "$C_RESET" "$*"; }
ok()   { printf '%s[+]%s %s\n' "$C_OK" "$C_RESET" "$*"; }
warn() { printf '%s[!]%s %s\n' "$C_WARN" "$C_RESET" "$*" >&2; }
err()  { printf '%s[x]%s %s\n' "$C_ERR" "$C_RESET" "$*" >&2; }
die()  { err "$*"; exit 1; }

usage() {
  awk 'NR == 1 { next } /^#/ { sub(/^# ?/, ""); print; next } { exit }' "$0"
}

# ---------------------------------------------------------------------------
# options
# ---------------------------------------------------------------------------
ASSUME_YES=0
DRY_RUN=0
INSTALL_AUDIO=1
INSTALL_TOOLKIT=1
INSTALL_SCREENCAPTURE=1
INSTALL_FONTS=1
USE_GURU=1
WITH_QUICKSHELL=0
WITH_WEBENGINE=0
WITH_SDDM=0
WITH_RUST=0
WITH_EXTRA_THEMES=0
WITH_OPTIONAL=0
ENABLE_SERVICES=0
ADD_GROUPS=0

while (($#)); do
  case "$1" in
    -y|--yes)              ASSUME_YES=1 ;;
    -n|--dry-run)          DRY_RUN=1 ;;
    --no-audio)            INSTALL_AUDIO=0 ;;
    --no-toolkit)          INSTALL_TOOLKIT=0 ;;
    --no-screencapture)    INSTALL_SCREENCAPTURE=0 ;;
    --no-fonts)            INSTALL_FONTS=0 ;;
    --no-guru)             USE_GURU=0 ;;
    --with-quickshell)     WITH_QUICKSHELL=1 ;;
    --with-webengine)      WITH_WEBENGINE=1 ;;
    --with-sddm)           WITH_SDDM=1 ;;
    --with-rust)           WITH_RUST=1 ;;
    --with-extra-themes)   WITH_EXTRA_THEMES=1 ;;
    --with-optional)       WITH_OPTIONAL=1 ;;
    --enable-services)     ENABLE_SERVICES=1 ;;
    --add-groups)          ADD_GROUPS=1 ;;
    -h|--help)             usage; exit 0 ;;
    *) die "unknown option: $1 (try --help)" ;;
  esac
  shift
done

# ---------------------------------------------------------------------------
# root and Gentoo checks
# ---------------------------------------------------------------------------
if ((EUID != 0)); then
  if command -v sudo >/dev/null 2>&1; then
    exec sudo -- "$0" "${ORIG_ARGS[@]}"
  elif command -v doas >/dev/null 2>&1; then
    exec doas -- "$0" "${ORIG_ARGS[@]}"
  else
    die "run as root: sudo ./$SCRIPT_NAME"
  fi
fi

[[ -f /etc/gentoo-release ]] || die "this script is for Gentoo only"
command -v emerge >/dev/null 2>&1 || die "emerge not found"
command -v portageq >/dev/null 2>&1 || die "portageq not found (sys-apps/portage)"

TARGET_USER=${SUDO_USER:-}
if [[ -z $TARGET_USER ]]; then
  TARGET_USER=$(logname 2>/dev/null || true)
fi

case "$(uname -m)" in
  x86_64)  PORTAGE_KEYWORD='~amd64' ;;
  aarch64) PORTAGE_KEYWORD='~arm64' ;;
  armv7l)  PORTAGE_KEYWORD='~arm' ;;
  *)       PORTAGE_KEYWORD="~$(uname -m)" ;;
esac

# ---------------------------------------------------------------------------
# package sets
# ---------------------------------------------------------------------------
CORE_PKGS=(
  # compositor and shell runtime
  gui-wm/niri
  app-crypt/mit-krb5          # qs-rs links libgssapi_krb5
  dev-libs/jemalloc
  dev-libs/wayland
  x11-libs/libxcb
  x11-libs/libdrm
  media-libs/mesa

  # Qt 6 modules loaded by the shell and by native dialogs
  dev-qt/qtbase
  dev-qt/qtdeclarative
  dev-qt/qtsvg
  dev-qt/qtwayland
  dev-qt/qt5compat
  dev-qt/qtimageformats
  dev-qt/qtmultimedia
  dev-qt/qttools
  dev-qt/qttranslations

  # KDE / Qt integration and theming
  kde-apps/kdialog
  kde-frameworks/breeze-icons
  kde-frameworks/frameworkintegration
  kde-frameworks/kconfig
  kde-frameworks/kirigami
  kde-frameworks/syntax-highlighting
  kde-plasma/kdecoration
  kde-plasma/plasma-integration
  gui-apps/qt6ct
  x11-themes/kvantum
  x11-themes/adw-gtk3       # GTK3/4 dark theme used by the shell and dialogs

  # session, desktop and CLI basics
  app-shells/fish
  app-shells/starship
  sys-apps/eza
  gnome-base/nautilus
  x11-terms/kitty
  gnome-base/gnome-keyring
  app-crypt/libsecret
  gnome-extra/nm-applet      # provides nm-connection-editor
  gnome-extra/polkit-gnome
  net-misc/networkmanager
  sys-auth/polkit
  sys-power/upower
  sys-power/power-profiles-daemon
  sys-apps/xdg-desktop-portal
  sys-apps/xdg-desktop-portal-gtk
  sys-apps/xdg-desktop-portal-gnome
  gui-libs/xdg-desktop-portal-wlr

  # clipboard, wallpaper, XWayland
  gui-apps/wl-clipboard
  gui-apps/grim
  gui-apps/slurp
  gui-apps/awww
  gui-apps/xwayland-satellite
  app-misc/cliphist
  gui-apps/wlsunset

  # base tools used by the shell scripts
  app-arch/unzip
  app-i18n/translate-shell
  app-misc/jq
  dev-lang/python
  dev-vcs/git
  net-misc/curl
  net-misc/rsync
  sys-apps/coreutils
  sys-apps/file
  sys-apps/ripgrep
  x11-misc/xdg-user-dirs
  x11-misc/xdg-utils

  # audio server library is also a hard dependency of the QML runtime
  media-video/pipewire
)

AUDIO_PKGS=(
  media-video/wireplumber
  media-sound/playerctl
  media-sound/pavucontrol
  media-sound/cava
  media-sound/easyeffects
  media-libs/lsp-plugins
  media-video/mpv
  mpv-plugin/mpv-mpris
  net-misc/yt-dlp
  net-misc/socat
  dev-lang/deno-bin
  kde-plasma/plasma-browser-integration
  media-sound/songrec
)

TOOLKIT_PKGS=(
  gui-apps/wtype
  x11-misc/ydotool
  sys-auth/fprintd
  app-misc/brightnessctl
  app-misc/ddcutil
  app-misc/geoclue
  gui-apps/swayidle
  gui-apps/swaylock
  sci-calculators/qalculate-qt
  net-wireless/blueman
  net-wireless/bluez
  sys-apps/mission-center
)

SCREENCAPTURE_PKGS=(
  gui-apps/swappy
  gui-apps/wf-recorder
  media-video/ffmpeg
  media-video/ffmpegthumbnailer
  xfce-base/tumbler
  media-gfx/imagemagick
  app-text/tesseract
  app-text/tessdata_fast
)

FONTS_PKGS=(
  media-libs/fontconfig
  media-fonts/dejavu
  media-fonts/liberation-fonts
  media-fonts/noto-emoji
  media-fonts/roboto
  media-fonts/jetbrains-mono
  x11-themes/adwaita-icon-theme
  x11-themes/hicolor-icon-theme
  x11-themes/papirus-icon-theme
  x11-themes/adw-gtk3
)

GURU_ATOMS=(
  gui-wm/niri
  gui-apps/awww
  gui-apps/wtype
  gui-apps/xwayland-satellite
  gui-apps/wlsunset
  app-misc/cliphist
  app-misc/brightnessctl
  x11-themes/adw-gtk3
  mpv-plugin/mpv-mpris
  media-sound/songrec
  sys-apps/mission-center
)

EXTRA_KEYWORD_ATOMS=(
  sys-apps/eza
  xfce-base/tumbler
  x11-themes/kvantum
)

if command -v rc-update >/dev/null 2>&1; then
  CORE_PKGS+=( sys-process/runit )   # runsvdir session supervisor on OpenRC
fi

if ((WITH_QUICKSHELL)); then
  CORE_PKGS+=( gui-apps/quickshell )
  GURU_ATOMS+=( gui-apps/quickshell )
fi

if ((WITH_WEBENGINE)); then
  CORE_PKGS+=( dev-qt/qtwebengine kde-plasma/layer-shell-qt )
fi

if ((WITH_SDDM)); then
  CORE_PKGS+=( gui-apps/sddm )
fi

OPTIONAL_PKGS=(
  net-vpn/cloudflare-warp
  sci-ml/ollama
  app-accessibility/whisper-cpp
  sys-apps/flatpak
)

if ((WITH_OPTIONAL)); then
  CORE_PKGS+=( "${OPTIONAL_PKGS[@]}" )
  GURU_ATOMS+=( app-accessibility/whisper-cpp )
  EXTRA_KEYWORD_ATOMS+=(
    net-vpn/cloudflare-warp
    sci-ml/ollama
    app-accessibility/whisper-cpp
  )
fi

if ((WITH_RUST)); then
  CORE_PKGS+=(
    dev-lang/rust
    dev-build/cmake
    dev-build/meson
    dev-build/ninja
    dev-util/pkgconf
    sys-devel/gcc
    dev-build/make
  )
fi

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------
FAILED=()
SKIPPED=()
CHANGED_KEYWORDS=0
KEYWORDS_FILE=''

atom_visible() {
  local out
  out=$(portageq best_visible / "$1" 2>/dev/null) || true
  [[ -n $out ]]
}

prepare_keywords_file() {
  local dir=/etc/portage/package.accept_keywords
  if [[ -d $dir ]]; then
    KEYWORDS_FILE="$dir/zz-inir-gentoo"
  elif [[ -e $dir ]]; then
    KEYWORDS_FILE=$dir
  else
    mkdir -p "$dir"
    KEYWORDS_FILE="$dir/zz-inir-gentoo"
  fi
  if [[ ! -e $KEYWORDS_FILE ]]; then
    printf '# Added by %s\n' "$SCRIPT_NAME" > "$KEYWORDS_FILE"
  fi
}

accept_keyword() {
  local atom=$1 keyword=${2:-$PORTAGE_KEYWORD}
  [[ -n $KEYWORDS_FILE ]] || prepare_keywords_file
  if grep -qxF "$atom $keyword" "$KEYWORDS_FILE" 2>/dev/null; then
    return 0
  fi
  printf '%s %s\n' "$atom" "$keyword" >> "$KEYWORDS_FILE"
  CHANGED_KEYWORDS=1
}

ensure_overlay() {
  local name=$1 uri=${2:-}

  if portageq get_repos / 2>/dev/null | tr ' ' '\n' | grep -qx "$name"; then
    return 0
  fi

  info "enabling the ::$name overlay"

  if command -v eselect >/dev/null 2>&1 && eselect repository list >/dev/null 2>&1; then
    if eselect repository list -i 2>/dev/null | grep -qE "(^|[[:space:]])${name}([[:space:]]|$)"; then
      if eselect repository enable "$name"; then
        emaint sync -r "$name" || warn "emaint sync -r $name failed"
        return 0
      fi
    elif [[ -n $uri ]] && eselect repository add "$name" git "$uri"; then
      emaint sync -r "$name" || warn "emaint sync -r $name failed"
      return 0
    fi
  fi

  [[ -n $uri ]] || return 1
  mkdir -p /etc/portage/repos.conf
  cat > "/etc/portage/repos.conf/${name}.conf" <<EOF
[$name]
location = /var/db/repos/$name
sync-type = git
sync-uri = $uri
EOF
  emaint sync -r "$name" || warn "emaint sync -r $name failed"
}

emerge_atoms() {
  local label=$1
  shift
  local -a avail=()
  local atom

  for atom in "$@"; do
    if atom_visible "$atom"; then
      avail+=("$atom")
    else
      warn "[$label] not available: $atom"
      SKIPPED+=("$atom")
    fi
  done

  ((${#avail[@]})) || return 0

  if ((DRY_RUN)); then
    info "[$label] would install: ${avail[*]}"
    return 0
  fi

  info "[$label] installing ${#avail[@]} package(s)"
  if emerge --noreplace --oneshot --quiet-build=y "${avail[@]}"; then
    ok "[$label] done"
    return 0
  fi

  warn "[$label] batch failed, retrying package by package"
  for atom in "${avail[@]}"; do
    if emerge --noreplace --oneshot --quiet-build=y "$atom"; then
      ok "[$label] $atom"
    else
      err "[$label] failed: $atom"
      FAILED+=("$atom")
    fi
  done
}

# Tesseract language models: Gentoo uses LINGUAS (ru/es/ja/zh), not the
# upstream tessdata package names. Keep this in package.env so a later
# @world update does not drop the models.
configure_tessdata_languages() {
  local envdir=/etc/portage/env
  local pkgdir=/etc/portage/package.env
  local envfile=inir-tessdata
  local entry="app-text/tessdata_fast $envfile"

  mkdir -p "$envdir"
  printf 'LINGUAS="ru es ja zh"\n' > "$envdir/$envfile"

  if [[ -d $pkgdir ]]; then
    grep -qxF "$entry" "$pkgdir/zz-inir-gentoo" 2>/dev/null \
      || printf '%s\n' "$entry" >> "$pkgdir/zz-inir-gentoo"
  elif [[ -e $pkgdir ]]; then
    grep -qxF "$entry" "$pkgdir" 2>/dev/null \
      || printf '%s\n' "$entry" >> "$pkgdir"
  else
    mkdir -p "$pkgdir"
    printf '%s\n' "$entry" > "$pkgdir/zz-inir-gentoo"
  fi
}

configure_gtk_theme() {
  local user=$TARGET_USER
  [[ -n $user && $user != root ]] || {
    warn "no desktop user; skipping GTK theme settings"
    return 0
  }

  local home uid group
  home=$(getent passwd "$user" | cut -d: -f6)
  uid=$(id -u "$user")
  group=$(id -gn "$user")
  [[ -n $home && -n $uid && -n $group ]] || return 0

  local version dir file
  for version in 3.0 4.0; do
    dir="$home/.config/gtk-$version"
    file="$dir/settings.ini"
    install -d -m 700 -o "$user" -g "$group" "$dir"

    if [[ -f $file ]]; then
      if grep -q '^gtk-theme-name=' "$file"; then
        sed -i 's/^gtk-theme-name=.*/gtk-theme-name=adw-gtk3-dark/' "$file"
      else
        printf 'gtk-theme-name=adw-gtk3-dark\n' >> "$file"
      fi
      grep -q '^gtk-application-prefer-dark-theme=1' "$file" \
        || printf 'gtk-application-prefer-dark-theme=1\n' >> "$file"
    else
      printf '[Settings]\ngtk-theme-name=adw-gtk3-dark\ngtk-application-prefer-dark-theme=1\n' > "$file"
    fi
    chown "$user:$group" "$file"
  done

  if command -v runuser >/dev/null 2>&1; then
    as_target_user() { runuser -u "$user" -- "$@"; }
  else
    as_target_user() { sudo -u "$user" -- "$@"; }
  fi

  if command -v gsettings >/dev/null 2>&1 && [[ -S /run/user/$uid/bus ]]; then
    as_target_user env XDG_RUNTIME_DIR="/run/user/$uid" \
      DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" \
      gsettings set org.gnome.desktop.interface gtk-theme adw-gtk3-dark 2>/dev/null \
      || warn "could not set gtk-theme via gsettings"
    as_target_user env XDG_RUNTIME_DIR="/run/user/$uid" \
      DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" \
      gsettings set org.gnome.desktop.interface color-scheme prefer-dark 2>/dev/null \
      || warn "could not set color-scheme via gsettings"
  else
    warn "gsettings session bus unavailable; GTK theme set in settings.ini only"
  fi
  ok "GTK theme adw-gtk3-dark configured"
}

install_gum() {
  command -v gum >/dev/null 2>&1 && { ok "gum already installed"; return 0; }

  local version=2.0.2
  local sha256=d842e06d93dbed90af48cb8dd10698db6f22e331fc40346bb37bbc753109edc2
  local arch url tmp
  case "$(uname -m)" in
    x86_64)        arch=x86_64 ;;
    aarch64|arm64) arch=arm64 ;;
    *) warn "no prebuilt gum for $(uname -m); skipping"; return 0 ;;
  esac

  url="https://github.com/charmbracelet/gum/releases/download/v${version}/gum_${version}_Linux_${arch}.tar.gz"
  tmp=$(mktemp -d)

  if curl -fsSL --retry 3 --connect-timeout 15 -o "$tmp/gum.tar.gz" "$url" \
      && { [[ $arch != x86_64 ]] || [[ $(sha256sum "$tmp/gum.tar.gz" | cut -d' ' -f1) == "$sha256" ]]; } \
      && tar -xzf "$tmp/gum.tar.gz" -C "$tmp" \
      && install -Dm755 "$tmp/gum_${version}_Linux_${arch}/gum" /usr/local/bin/gum; then
    ok "gum $version installed to /usr/local/bin/gum"
  else
    warn "could not install gum; the installer falls back to plain text"
  fi
  rm -rf "$tmp"
}

install_material_symbols() {
  local dir=/usr/local/share/fonts/inir
  local set url file
  mkdir -p "$dir"
  for set in Rounded Outlined Sharp; do
    file="$dir/MaterialSymbols${set}.ttf"
    [[ -s $file ]] && continue
    url="https://raw.githubusercontent.com/google/material-design-icons/master/variablefont/MaterialSymbols${set}%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf"
    if curl -fsSL --retry 3 --connect-timeout 15 -o "$file" "$url"; then
      ok "Material Symbols $set installed"
    else
      rm -f "$file"
      warn "could not download Material Symbols $set"
    fi
  done
  command -v fc-cache >/dev/null 2>&1 && fc-cache -f "$dir" >/dev/null 2>&1 || true
}

install_jetbrains_nerd() {
  if command -v fc-list >/dev/null 2>&1 && fc-list 2>/dev/null | grep -qi 'JetBrainsMono Nerd'; then
    ok "JetBrains Mono Nerd Font already installed"
    return 0
  fi

  local tmp dir=/usr/local/share/fonts/inir/JetBrainsMono
  tmp=$(mktemp -d)
  if curl -fsSL --retry 3 --connect-timeout 15 \
      -o "$tmp/JetBrainsMono.zip" \
      "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip"; then
    mkdir -p "$dir"
    if unzip -o -q "$tmp/JetBrainsMono.zip" -d "$dir"; then
      command -v fc-cache >/dev/null 2>&1 && fc-cache -f "$dir" >/dev/null 2>&1 || true
      ok "JetBrains Mono Nerd Font installed"
    else
      warn "could not unpack JetBrains Mono Nerd Font"
    fi
  else
    warn "could not download JetBrains Mono Nerd Font"
  fi
  rm -rf "$tmp"
}

install_capitaine_cursors() {
  local tmp url
  tmp=$(mktemp -d)
  url=$(curl -fsSL --connect-timeout 15 \
    https://api.github.com/repos/sainnhe/capitaine-cursors/releases/latest 2>/dev/null \
    | jq -r '.assets[]? | select(.name | test("\\.zip$")) | .browser_download_url' 2>/dev/null \
    | head -n1)
  if [[ -n $url ]] && curl -fsSL --retry 3 --connect-timeout 15 -o "$tmp/capitaine.zip" "$url" \
      && unzip -o -q "$tmp/capitaine.zip" -d "$tmp"; then
    mkdir -p /usr/share/icons
    local d
    for d in "$tmp"/*; do
      [[ -d $d ]] && cp -a "$d" /usr/share/icons/
    done
    ok "Capitaine cursors installed"
  else
    warn "could not install Capitaine cursors"
  fi
  rm -rf "$tmp"
}

install_whitesur_icons() {
  local tmp
  tmp=$(mktemp -d)
  if curl -fsSL --retry 3 --connect-timeout 15 \
      -o "$tmp/whitesur.tar.gz" \
      "https://github.com/vinceliuice/WhiteSur-icon-theme/archive/refs/heads/master.tar.gz" \
      && tar -xzf "$tmp/whitesur.tar.gz" -C "$tmp"; then
    mkdir -p /usr/share/icons
    cp -a "$tmp"/WhiteSur-icon-theme-master/src/WhiteSur* /usr/share/icons/ 2>/dev/null || true
    ok "WhiteSur icons installed"
  else
    warn "could not install WhiteSur icons"
  fi
  rm -rf "$tmp"
}

enable_services() {
  local svc
  if command -v rc-update >/dev/null 2>&1; then
    for svc in polkit power-profiles-daemon bluetooth ydotool; do
      [[ -x /etc/init.d/$svc ]] || continue
      rc-update add "$svc" default >/dev/null 2>&1 || true
      rc-service "$svc" status >/dev/null 2>&1 || rc-service "$svc" start >/dev/null 2>&1 || true
      ok "OpenRC service enabled: $svc"
    done
    mkdir -p /etc/modules-load.d
    printf 'i2c-dev\n' > /etc/modules-load.d/i2c-dev.conf
    modprobe i2c-dev 2>/dev/null || true
    warn "NetworkManager was NOT enabled automatically (it can cut an existing dhcpcd link)"
  elif command -v systemctl >/dev/null 2>&1; then
    systemctl enable --now polkit power-profiles-daemon bluetooth 2>/dev/null || true
    ok "systemd services enabled"
  else
    warn "no supported init system found; enable services yourself"
  fi
}

add_user_groups() {
  [[ -n $TARGET_USER && $TARGET_USER != root ]] || {
    warn "cannot determine the desktop user; add groups manually"
    return 0
  }
  local g
  for g in input video render plugdev audio ydotool; do
    if getent group "$g" >/dev/null 2>&1; then
      if usermod -aG "$g" "$TARGET_USER" 2>/dev/null; then
        ok "added $TARGET_USER to $g"
      fi
    fi
  done
  warn "log out and back in for the new groups to take effect"
}

# ---------------------------------------------------------------------------
# confirmation and dry-run
# ---------------------------------------------------------------------------
if ((!ASSUME_YES && !DRY_RUN)) && [[ -t 0 ]]; then
  printf 'Install iNiR / Ginir-rs dependencies on Gentoo? [Y/n] '
  read -r answer
  case ${answer:-Y} in
    [Yy]*) ;;
    *) exit 0 ;;
  esac
fi

if ((DRY_RUN)); then
  info "dry run: no changes will be made"
  printf '\n  core:           %s\n' "${CORE_PKGS[*]}"
  ((INSTALL_AUDIO))         && printf '  audio:          %s\n' "${AUDIO_PKGS[*]}"
  ((INSTALL_TOOLKIT))       && printf '  toolkit:        %s\n' "${TOOLKIT_PKGS[*]}"
  ((INSTALL_SCREENCAPTURE)) && printf '  screencapture:  %s\n' "${SCREENCAPTURE_PKGS[*]}"
  ((INSTALL_FONTS))         && printf '  fonts:          %s\n' "${FONTS_PKGS[*]}"
  ((USE_GURU))              && printf '  ::guru atoms:   %s\n' "${GURU_ATOMS[*]}"
  printf '\n  providers:      gum, Material Symbols, JetBrains Mono Nerd Font\n'
  ((WITH_EXTRA_THEMES)) && printf '  extra themes:   Darkly, Capitaine cursors, WhiteSur icons\n'
  ((WITH_OPTIONAL))     && printf '  optional:       %s\n' "${OPTIONAL_PKGS[*]}"
  exit 0
fi

# ---------------------------------------------------------------------------
# overlays and keywords
# ---------------------------------------------------------------------------
prepare_keywords_file

if ((USE_GURU)); then
  ensure_overlay guru "https://github.com/gentoo-mirror/guru.git" \
    || warn "could not enable ::guru; some packages will be skipped"

  for atom in "${GURU_ATOMS[@]}"; do
    accept_keyword "$atom"
  done
fi

for atom in "${EXTRA_KEYWORD_ATOMS[@]}"; do
  accept_keyword "$atom"
done

if ((WITH_EXTRA_THEMES)); then
  ensure_overlay huang "https://github.com/jwihardi/huang.git" \
    || warn "could not enable ::huang; Darkly and hyprpicker will be skipped"
  accept_keyword kde-misc/darkly
  accept_keyword gui-apps/hyprpicker
fi

if ((CHANGED_KEYWORDS)); then
  ok "wrote keywords to $KEYWORDS_FILE"
fi

# If the overlay was already present but stale, make one sync attempt when a
# key atom is still invisible.
if ((USE_GURU)) && ! atom_visible gui-wm/niri; then
  info "syncing ::guru"
  emaint sync -r guru >/dev/null 2>&1 || warn "emaint sync -r guru failed"
fi

# ---------------------------------------------------------------------------
# install
# ---------------------------------------------------------------------------
info "installing core packages"
emerge_atoms core "${CORE_PKGS[@]}"

if ((INSTALL_AUDIO)); then
  info "installing audio packages"
  emerge_atoms audio "${AUDIO_PKGS[@]}"
fi

if ((INSTALL_TOOLKIT)); then
  info "installing toolkit packages"
  emerge_atoms toolkit "${TOOLKIT_PKGS[@]}"
fi

if ((INSTALL_SCREENCAPTURE)); then
  info "installing screenshot and OCR packages"
  emerge_atoms screencapture "${SCREENCAPTURE_PKGS[@]}"
  configure_tessdata_languages
  if atom_visible app-text/tessdata_fast; then
    emerge --oneshot --newuse --quiet-build=y app-text/tessdata_fast \
      || warn "could not rebuild tessdata_fast with the extra languages"
  fi
fi

if ((INSTALL_FONTS)); then
  info "installing fonts and themes"
  emerge_atoms fonts "${FONTS_PKGS[@]}"
fi

if ((WITH_EXTRA_THEMES)); then
  info "installing extra themes"
  emerge_atoms extra-themes kde-misc/darkly gui-apps/hyprpicker
  install_capitaine_cursors
  install_whitesur_icons
fi

# Things without an ebuild in ::gentoo/::guru.
info "installing helper binaries and fonts"
install_gum
install_material_symbols
install_jetbrains_nerd
configure_gtk_theme

if ((ENABLE_SERVICES)); then
  info "enabling services"
  enable_services
fi

if ((ADD_GROUPS)); then
  info "adding the desktop user to device groups"
  add_user_groups
fi

# ---------------------------------------------------------------------------
# summary
# ---------------------------------------------------------------------------
printf '\n'
if ((${#FAILED[@]})); then
  err "failed packages:"
  printf '  - %s\n' "${FAILED[@]}" >&2
else
  ok "all requested packages installed"
fi

if ((${#SKIPPED[@]})); then
  warn "skipped (not visible in the enabled overlays):"
  printf '  - %s\n' "${SKIPPED[@]}" >&2
fi

info "checking important commands"
for cmd in niri fish gum cliphist grim slurp wl-copy awww-daemon tesseract magick qalc playerctl mpv yt-dlp; do
  if command -v "$cmd" >/dev/null 2>&1; then
    ok "$cmd"
  else
    warn "$cmd not found"
  fi
done

cat <<'EOF'

Notes:
  - gowall has no Gentoo ebuild; iNiR's wallpaper theming uses awww instead.
  - hyprpicker and Darkly are installed only with --with-extra-themes.
  - The prebuilt qs-rs runtime already carries its own Qt/KF6; a system
    gui-apps/quickshell is only needed for a source Quickshell build.

Next steps:
  1. Restart the computer (or at least log out and back in) so Niri's
     environment and any new group memberships are loaded.
  2. In the iNiR checkout run:  ./setup install --skip-deps
  3. If you use OpenRC and want the usual desktop services:
       rc-update add polkit default && rc-service polkit start
       rc-update add power-profiles-daemon default && rc-service power-profiles-daemon start
       rc-update add bluetooth default && rc-service bluetooth start
     NetworkManager is installed but not enabled, so an existing dhcpcd
     connection is not cut. Switch deliberately when ready:
       rc-update del dhcpcd default
       rc-update add NetworkManager default
       rc-service NetworkManager start
  4. If you run the prebuilt qs-rs runtime, gui-apps/quickshell is not
     needed. Use --with-quickshell only for a source Quickshell build.

EOF
