<p align="center">
  <img src="../images/readme/mark.svg" alt="" width="56">
</p>

<h1 align="center">Ginir-rs</h1>

<p align="center">
  Rust-first слой темизации и бэкенда для <a href="https://github.com/snowarch/iNiR">iNiR</a> на Gentoo.<br>
  Разберите акцент один раз — и держите в синхроне все приложения.
</p>

<p align="center">
  <a href="https://github.com/Gergish001/Ginir-rs/actions/workflows/rust.yml"><img src="https://github.com/Gergish001/Ginir-rs/actions/workflows/rust.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/Gergish001/Ginir-rs/stargazers"><img src="https://img.shields.io/github/stars/Gergish001/Ginir-rs?style=flat-square&color=111111" alt="Stars"></a>
  <a href="https://github.com/Gergish001/Ginir-rs/releases"><img src="https://img.shields.io/badge/version-0.1.0-111111?style=flat-square" alt="Version 0.1.0"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-111111?style=flat-square" alt="GPL-3.0"></a>
  <a href="../../rust-toolchain.toml"><img src="https://img.shields.io/badge/rust-1.85%2B-111111?style=flat-square&logo=rust&logoColor=white" alt="Rust 1.85+"></a>
</p>

<p align="center">
  <a href="#targets">Цели</a> ·
  <a href="#build-and-install">Установка</a> ·
  <a href="#usage">Использование</a> ·
  <a href="#inir-integration">Интеграция с iNiR</a> ·
  <a href="https://github.com/snowarch/iNiR">Upstream</a> ·
  <a href="https://github.com/Gergish001/Ginir-rs/issues">Issues</a>
</p>

<p align="center">
  <sub>
    <a href="../../README.md">English</a> ·
    <a href="README.ru.md">Русский</a> ·
    <a href="TRANSLATING.md">помочь с переводом</a>
  </sub>
</p>

Ginir-rs — новый Rust-проект, выросший из iNiR; репозиторий на GitHub помечен как
форк только потому, что так он был создан. Проект покрывает математику палитры,
парсинг и преобразование акцента, а также генераторы, которые раздают полученный
акцент другим приложениям. Оригинальное QML-дерево iNiR сохранено без изменений в
[`upstream/inir/`](../../upstream/inir/) как эталон.

> **Что здесь значит «Rust-first».** Логика темизации и бэкенда — чтение палитры
> iNiR, преобразование акцента, запись конфигов приложений, перезагрузка
> приложения — написана на Rust (`inir-theme`). QML-виджеты iNiR не
> переписываются: они по-прежнему работают на `qs-rs`, отдельном upstream-проекте.
> Переписать эти виджеты на Rust — значит сделать новую оболочку, а не порт.

## Цели <a id="targets"></a>

Каждая цель читает текущий акцент iNiR и записывает его в другое приложение,
применяя собственное преобразование.

| id | Куда пишет | Преобразование |
|---|---|---|
| `herdr` | `~/.config/herdr/config.toml` `[theme.custom]` | точный акцент iNiR, более тёмные поверхности |
| `opencode` | `~/.config/opencode/themes/inir.json` `m3Tertiary` | точный акцент iNiR (патч активной темы) |
| `niri` | Niri `focus-ring active-color` в `config.kdl` + `config.d/*.kdl` | −20% светлоты, насыщенность ×1.34 |
| `fetch3d` | `~/.config/fetch/config` `label_color` | −10% светлоты (3D fetch, **не** fastfetch) |
| `launchers` | `~/.local/bin/cmx`, `~/.local/bin/lavax` | ближайший ANSI / акцент + teal |

`fetch3d` — это кастомный 3D-fetch. **fastfetch, btop и yazi этот проект не
трогает** и не добавляет им патчи или конфиги.

## Сборка и установка <a id="build-and-install"></a>

```sh
cargo build --release
install -m755 target/release/inir-theme ~/.local/bin/inir-theme
# или
./scripts/install.sh
```

Нужен Rust 1.85+; Qt и Python не нужны.

## Использование <a id="usage"></a>

```sh
inir-theme list
inir-theme doctor
inir-theme apply --target herdr
inir-theme apply --all --dry-run
inir-theme apply --all
```

Источник выбирается автоматически: сгенерированные iNiR `app-palette.json` +
`terminal.json`, с откатом на `~/.config/kitty/theme.conf`. Можно переопределить:

```sh
inir-theme apply --all \
  --app-palette ~/.local/state/quickshell/user/generated/app-palette.json \
  --terminal-json ~/.local/state/quickshell/user/generated/terminal.json \
  --accent '#b78cda'
```

Каждая запись сохраняется в бэкап в `~/gentoo-opt-backups/`. `--dry-run` только
показывает, что изменится.

## Интеграция с iNiR <a id="inir-integration"></a>

`integration/inir/` содержит модули цветов, которые вызывают Rust-бинарь, и их
манифесты целей. Установить в живую iNiR:

```sh
./scripts/install.sh                 # собрать и поставить ~/.local/bin/inir-theme
./scripts/install-inir-modules.sh    # скопировать модули/цели, сделав бэкапы
inir theme doctor
```

| модуль | манифест цели | Rust-цель |
|---|---|---|
| `05-niri.sh` | `niri.json` | `niri` |
| `95-herdr.sh` | `herdr.json` | `herdr` |
| `96-opencode-accent.sh` | `opencode-accent.json` | `opencode` |
| `97-fetch3d.sh` | `fetch3d.json` | `fetch3d` |
| `98-launchers.sh` | `launchers.json` | `launchers` |

Модули читают `app-palette.json` и `terminal.json`, которые iNiR записывает до
запуска модулей темизации, поэтому гонки с kitty-целью нет.

## Структура

```
crates/inir-theme-core/         парсинг акцента, математика цвета, палитра, цели
crates/inir-theme-cli/          бинарь inir-theme
crates/inir-theme-core/assets/  шаблон OpenCode (производный от iNiR)
integration/inir/               модули iNiR + манифесты целей (дополнение)
patches/                        внешние C-патчи вне Rust
legacy/                         исходные реализации на Python/Bash
scripts/install.sh              локальная установка бинаря
scripts/install-inir-modules.sh установка модулей в живую iNiR
upstream/inir/                  эталонное QML-дерево iNiR без изменений
```

## Gentoo

Сейчас Rust-бинарь ставится через `cargo`/`scripts/install.sh`. Для полного стека
нужен Gentoo-overlay:

- `gui-apps/qs-rs` (Rust + Qt 6/LayerShellQt или готовый Qt-бандл)
- `dev-util/inir-accel` (Rust)
- `dev-util/inir-cli` (Go)
- `gui-apps/inir` (QML-пейлоад + модули + сервис runit/OpenRC)

Это отдельные upstream-репозитории; см. `../../packaging/gentoo/README.md`.
Полного ebuild для iNiR пока нет: сложная часть — упаковка qs-rs/Qt, а не этот CLI.

## Связь с upstream и лицензии

- iNiR: GPL-3.0
- qs-rs: LGPL-3.0
- inir-accel: LGPL-3.0
- inir-cli: GPL-3.0
- этот проект: **GPL-3.0**

Шаблон OpenCode в `crates/inir-theme-core/assets/` произведён от сгенерированной
темы iNiR и остаётся под лицензией iNiR. Патч fetch относится к отдельному
исходнику 3D `fetch`.

## Планы

1. Перенести больше потребителей акцента (lazygit, акценты GTK/Qt, другие
   терминалы) в `inir-theme-core`, каждый отдельной целью.
2. Перенести генераторы в upstream `inir-accel`, чтобы оболочка получала один
   Rust-бинарь вместо Python-хелпера и Bash-модулей.
3. Добавить цветовую тему/плагин для Hub, чтобы цели ставились без форка.
4. Выпустить Gentoo-overlay для `inir-theme`, `qs-rs`, `inir-accel`, `inir-cli`
   и iNiR.
