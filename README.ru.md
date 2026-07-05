<!--
SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

<a id="top"></a>

<div align="center">

# twc-rs

**Быстрый CLI из одного бинарника _и_ интерактивная TUI-панель для [Timeweb Cloud](https://timeweb.cloud).**

Серверы, базы данных, S3, Kubernetes, балансировщики, домены, файрволы и не только —
всё из одного нативного бинарника. Без Python, без `pip`, без virtualenv.

[![crates.io](https://img.shields.io/crates/v/twc-rs.svg?logo=rust&color=fc8d62)](https://crates.io/crates/twc-rs)
[![downloads](https://img.shields.io/crates/d/twc-rs.svg?color=brightgreen)](https://crates.io/crates/twc-rs)
[![CI](https://github.com/RAprogramm/twc-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/ci.yml)
[![Security](https://github.com/RAprogramm/twc-rs/actions/workflows/security.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/security.yml)
[![release-plz](https://github.com/RAprogramm/twc-rs/actions/workflows/release-plz.yml/badge.svg)](https://github.com/RAprogramm/twc-rs/actions/workflows/release-plz.yml)
[![license](https://img.shields.io/crates/l/twc-rs.svg?color=blue)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.96-blue.svg?logo=rust)](Cargo.toml)
[![platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-informational?logo=linux)](#supported-platforms)

[English](README.md) · **Русский**

</div>

---

## Содержание

- [Зачем twc-rs](#why-twc-rs)
- [Панель управления](#the-dashboard)
- [Установка](#install)
  - [Прямые ссылки на скачивание](#direct-downloads)
  - [Поддерживаемые платформы](#supported-platforms)
- [Аутентификация](#authenticate)
- [Использование](#usage)
- [Автодополнение оболочки](#shell-completions)
- [Бенчмарки](#benchmarks)
- [Сборка из исходников](#building-from-source)
- [Лицензия](#license)

## Зачем twc-rs

<a id="why-twc-rs"></a>

Официальный Timeweb CLI ([`timeweb-cloud/twc`](https://github.com/timeweb-cloud/twc)) —
это приложение на Python. `twc-rs` покрывает тот же набор команд и обходит его по
скорости, размеру и удобству — каждое число ниже **измерено, а не оценено на глаз**
(полный отчёт и воспроизводимый бенчмарк — в [docs/COMPARISON.md](docs/COMPARISON.md)).

| | `twc-rs` (Rust) | Официальный `twc` (Python) |
|---|---|---|
| Холодный старт (`--version`) | **2.3 мс** | 357 мс |
| Холодный старт (`--help`) | **2.1 мс** | 347 мс |
| Пиковая память (RSS) | **13.7 МБ** | 59.2 МБ |
| Размер | 1 статический бинарник, 15 МБ после strip | 33 МБ пакетов + интерпретатор Python |
| Рантайм-зависимости | только системная libc | Python + 15 PyPI-пакетов |
| Покрытие команд | почти полный паритет | базовое |
| Интерактивная панель | **да** — полноценный TUI с live-метриками | нет |
| Создание / удаление из панели | **да** | нет |
| Автодополнение | bash, zsh, fish, powershell, elvish, **nushell** | bash, zsh, fish, powershell |
| Форматы вывода | table, json, yaml, quiet | default, raw, json, yaml |
| Профили (мультиаккаунт) | да (`--profile`, переключение в TUI) | да |
| Языки | **английский + русский** (TUI и CLI) | только английский |

Замеры на AMD Ryzen AI MAX+ 395 (Linux 7.1, rustc 1.96) против `twc-cli` v2.15.2,
по 50 запусков. Python-инструмент тратит ~350 мс на старт интерпретатора и импорты
ещё до выполнения кода приложения.

<p align="right"><a href="#top">↑ наверх</a></p>

## Панель управления

<a id="the-dashboard"></a>

Главная фича, которой нет у Python-CLI: живой TUI в стиле k9s (`twc-rs dashboard`).

| Клавиша | Действие |
|---|---|
| `h` / `l` | переключение вкладок ресурсов |
| `j` / `k` | перемещение по списку |
| `Enter` | меню действий / вход в ресурс |
| `n` | создать новый ресурс (где поддерживается) |
| `/` | фильтр текущего списка |
| `Ctrl+K` | палитра команд — действия, тема, язык, смена профиля |
| `?` | справка |
| `Q` | выход |

- Контекстное меню действий по ресурсу (перезагрузка / выключение / клон / удаление)
  с подтверждением для необратимых операций.
- Создание ресурсов и **смена профиля аккаунта** прямо из панели.
- Live-метрики по каждому ресурсу (CPU / RAM / сеть, спарклайны), запросы вне
  UI-потока — ввод никогда не блокируется на сети.
- Вход в проект со списком его ресурсов; живой лог событий.
- Настраиваемая раскладка, скрытие пустых вкладок, 4 true-color темы и EN/RU —
  всё сохраняется в конфиг.

<p align="right"><a href="#top">↑ наверх</a></p>

## Установка

<a id="install"></a>

Выберите канал под свою платформу. Во всех каналах интерактивная TUI-панель
**включена по умолчанию**. Для лёгкой headless-сборки без неё:
`cargo install twc-rs --no-default-features --features auth`.

| Канал | Команда |
|---|---|
| **crates.io** | `cargo install twc-rs` |
| **Установщик** (Linux/macOS) | `curl -fsSL https://raw.githubusercontent.com/RAprogramm/twc-rs/main/install.sh \| sh` |
| **Arch (AUR)** | `paru -S twc-rs-bin` |
| **Debian/Ubuntu** | `sudo apt install ./twc-rs_<ver>_amd64.deb` |
| **Releases** | скачайте архив из [Releases](https://github.com/RAprogramm/twc-rs/releases), проверьте `.sha256`, положите `twc-rs` в `PATH` |

### Прямые ссылки на скачивание

<a id="direct-downloads"></a>

Готовые архивы с контрольными суммами прикреплены к
**[последнему релизу](https://github.com/RAprogramm/twc-rs/releases/latest)** —
выберите свою платформу (у каждого архива есть парный `.sha256`):

| Платформа | Архив |
|---|---|
| 🐧 Linux `x86_64` | [`twc-rs-*-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🐧 Linux `aarch64` | [`twc-rs-*-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🍎 macOS Intel | [`twc-rs-*-x86_64-apple-darwin.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🍎 macOS Apple Silicon | [`twc-rs-*-aarch64-apple-darwin.tar.gz`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 🪟 Windows `x86_64` | [`twc-rs-*-x86_64-pc-windows-msvc.zip`](https://github.com/RAprogramm/twc-rs/releases/latest) |
| 📦 Debian/Ubuntu | [`twc-rs_*_amd64.deb`](https://github.com/RAprogramm/twc-rs/releases/latest) |

Установщик одной строкой сам определяет ОС/архитектуру, качает нужный архив из
последнего релиза и ставит в `~/.local/bin` (или `/usr/local/bin`, если доступно
на запись). `.deb`-пакет прикрепляется к каждому релизу автоматически.

> `twc-rs` **отсутствует** в официальных репозиториях Debian/Ubuntu
> (`apt install twc-rs`) и Arch (`pacman -S twc-rs`) — для этого нужен статус
> сопровождающего в дистрибутиве. Используйте AUR-пакет, `.deb`, установщик или
> `cargo install`.

### Поддерживаемые платформы

<a id="supported-platforms"></a>

Каждый релиз содержит готовые бинарники с контрольными суммами для:

| ОС | Архитектуры |
|---|---|
| Linux (glibc) | `x86_64`, `aarch64` |
| macOS | `x86_64` (Intel), `aarch64` (Apple Silicon) |
| Windows | `x86_64` |

<p align="right"><a href="#top">↑ наверх</a></p>

## Аутентификация

<a id="authenticate"></a>

```sh
twc-rs auth flow                       # мастер в браузере, токен в системном keyring
# или
twc-rs config set-token --token <TOKEN>
```

Токен берётся из системного keyring, конфиг-файла, `--token` или переменной
окружения `TWC_TOKEN` — именно в таком порядке. Несколько аккаунтов
поддерживаются через именованные профили:

```sh
twc-rs config set-token --profile staging --token <TOKEN>
twc-rs --profile staging server list
```

<p align="right"><a href="#top">↑ наверх</a></p>

## Использование

<a id="usage"></a>

Каждый тип ресурса — это подкоманда; `twc-rs <группа> --help` покажет её действия
и флаги.

```sh
twc-rs server list                        # список серверов
twc-rs server info --id 12345             # детали сервера
twc-rs database list -f json              # вывод в JSON
twc-rs ssh attach --server 12345 --key 42 # прикрепить SSH-ключ к серверу
twc-rs project resources --id 678         # ресурсы внутри проекта
twc-rs dashboard                          # интерактивный TUI (в стиле k9s)
```

Группы ресурсов: `server`, `database`, `s3`, `kubernetes`, `registry`,
`balancer`, `domain`, `firewall`, `apps`, `image`, `ip`, `vpc`, `ssh`,
`project`, `account`. Полное покрытие команд относительно официального CLI — в
[docs/COMPARISON.md](docs/COMPARISON.md).

Глобальные флаги:

| Флаг | Env | Значение |
|---|---|---|
| `-f, --format <table\|json\|yaml\|quiet>` | `TWC_OUTPUT` | формат вывода (по умолчанию `table`) |
| `-t, --token <TOKEN>` | `TWC_TOKEN` | переопределение API-токена |
| `--profile <NAME>` | `TWC_PROFILE` | именованный профиль для мультиаккаунта |

<p align="right"><a href="#top">↑ наверх</a></p>

## Автодополнение оболочки

<a id="shell-completions"></a>

```sh
twc-rs completions nushell > ~/.config/nushell/completions/twc-rs.nu
twc-rs completions zsh     > ~/.zfunc/_twc-rs
twc-rs completions bash    > /etc/bash_completion.d/twc-rs
```

Поддерживаемые оболочки: `bash`, `zsh`, `fish`, `powershell`, `elvish`,
`nushell`. AUR-пакет ставит автодополнение для `bash`, `zsh`, `fish` и `nushell`
в стандартные системные каталоги — оно работает сразу после установки.


<p align="right"><a href="#top">↑ наверх</a></p>

## Бенчмарки

<a id="benchmarks"></a>

Все заявления о производительности воспроизводимы.
[docs/COMPARISON.md](docs/COMPARISON.md) описывает окружение и методику; повторить:

```sh
cargo build --release --features tui
python3 -m venv /tmp/twcbench && /tmp/twcbench/bin/pip install twc-cli
benches/compare.sh ./target/release/twc-rs /tmp/twcbench/bin/twc
```

Через CI? Workflow **[Benchmarks](../../actions/workflows/benchmarks.yml)**
запускает то же сравнение по требованию (Actions → Benchmarks → *Run workflow*) и
печатает таблицу в summary прогона. Только вручную — никогда на push, поэтому не
замедляет обычный CI.

<p align="right"><a href="#top">↑ наверх</a></p>

## Сборка из исходников

<a id="building-from-source"></a>

```sh
git clone https://github.com/RAprogramm/twc-rs && cd twc-rs
cargo build --release                   # полный бинарник (TUI включён по умолчанию)
cargo install --path .                  # установить из чекаута
# headless, без TUI:
cargo build --release --no-default-features --features auth
```

Требуется Rust **1.96+**. Крейт линтуется под `clippy` pedantic + nursery, а SDK
генерируется из официальной OpenAPI-спеки через
[`timeweb-rs`](https://crates.io/crates/timeweb-rs).

<p align="right"><a href="#top">↑ наверх</a></p>

## Лицензия

<a id="license"></a>

MIT © RAprogramm

<p align="right"><a href="#top">↑ наверх</a></p>
