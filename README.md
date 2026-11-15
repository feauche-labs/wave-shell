# Wave Shell

**Русский** · [English](README.en.md)

Лёгкий SSH- и SFTP-клиент для тех, кто администрирует много серверов: вложенные папки, флаги стран, терминал,
который не мешает, и секреты, которые не покидают ваш компьютер. macOS, Windows и Linux.

## Скачать

Нажмите на свою систему, и файл начнёт скачиваться. Всегда берётся последняя версия.

<p align="center">
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-macOS-Apple-Silicon.dmg"><img alt="Скачать macOS · Apple Silicon" src="https://img.shields.io/badge/%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C-macOS%20%C2%B7%20Apple%20Silicon-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-macOS-Intel.dmg"><img alt="Скачать macOS · Intel" src="https://img.shields.io/badge/%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C-macOS%20%C2%B7%20Intel-555555?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Windows-x64-setup.exe"><img alt="Скачать Windows 10/11" src="https://img.shields.io/badge/%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C-Windows%2010/11-0078D4?style=for-the-badge&logo=windows&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Linux-x64.AppImage"><img alt="Скачать Linux · AppImage" src="https://img.shields.io/badge/%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C-Linux%20%C2%B7%20AppImage-E95420?style=for-the-badge&logo=linux&logoColor=white"></a>
  <a href="https://github.com/feauche-labs/wave-shell/releases/latest/download/Wave-Shell-Linux-x64.deb"><img alt="Скачать Linux · .deb" src="https://img.shields.io/badge/%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C-Linux%20%C2%B7%20.deb-A81D33?style=for-the-badge&logo=debian&logoColor=white"></a>
</p>

Другие версии и контрольные суммы (`SHA256SUMS.txt`) — на странице
[Releases](https://github.com/feauche-labs/wave-shell/releases).

Сборки не подписаны платным сертификатом, поэтому система предупредит при первом запуске:

- **macOS.** Перетащите приложение в «Программы». При первом запуске — правой кнопкой → «Открыть» → «Открыть».
  Если macOS пишет, что приложение повреждено:
  `xattr -dr com.apple.quarantine "/Applications/Wave Shell.app"`.
- **Windows.** В окне SmartScreen — «Подробнее» → «Выполнить в любом случае».
- **Linux.** `chmod +x` на AppImage и запуск, либо `sudo apt install ./файл.deb`.

## Что умеет

- **Серверы по месту.** Вложенные папки, флаг страны у каждого хоста, поиск, импорт из `~/.ssh/config`.
- **Терминал.** Вкладки (перетаскиваются и выносятся в отдельное окно), 15 тем, локальный терминал, переподключение
  при обрыве, «Команда при входе» и tmux одной кнопкой, уведомление, когда долгая команда закончилась.
- **Файлы (SFTP).** Очередь передач с докачкой и паузой, перетаскивание из Finder, просмотр текстов и логов,
  открытие файла в системном приложении, передача между серверами.
- **Состояние серверов без агента.** CPU по ядрам, память, диск, сеть, процессы — через уже открытое SSH-соединение.
- **Docker.** Контейнеры, Compose, образы, тома, сети, события и логи на всех подключённых серверах.
- **Домены.** Сроки SSL-сертификатов (Let's Encrypt, nginx, Apache, Caddy) со всех серверов.
- **Проброс портов.** Локальный, обратный и SOCKS5, автозапуск при подключении.
- **Ключи.** Создание ключа Ed25519 и установка на сервер одной кнопкой, как `ssh-copy-id`.
- **Сниппеты.** Команды с папками, запуск на одном или сразу на нескольких серверах.
- **Безопасность.** Зашифрованное хранилище (Argon2id + XChaCha20-Poly1305), ключ — в связке ключей системы.
  Смена ключа хоста блокирует подключение. Без аккаунтов и облака. Зашифрованная резервная копия.
- **Клавиатура.** Палитра команд <kbd>⌘K</kbd> / <kbd>Ctrl K</kbd>, все сочетания — по <kbd>?</kbd>.
- Интерфейс на русском и английском, светлая и тёмная темы, значок в строке меню macOS.

## Разработка

Нужны Rust (stable), Node с pnpm и [just](https://github.com/casey/just).

```sh
pnpm install
pnpm tauri dev                  # запустить приложение
cargo test --workspace          # тесты Rust
pnpm -C ui check && pnpm -C ui test
just e2e                        # интеграционные тесты на OpenSSH в Docker
```

Структура:

```
crates/store   SQLite, дерево папок, наследование настроек
crates/vault   ключи, шифрование секретов, формат резервной копии
crates/ssh     клиент на russh: вход, ключи хостов, PTY, проброс, прыжки
crates/sftp    операции SFTP и движок передач
src-tauri      оболочка приложения: команды, каналы, права
ui             интерфейс на Svelte 5
tests/docker   серверы OpenSSH для интеграционных тестов
docs           план, контракты и решения (ADR) исходного проекта
```

## Выпуск версии

Версии выпускает [release-please](https://github.com/googleapis/release-please), руками номера и метки не ставятся.

1. Коммиты в `main` пишутся с приставкой: `feat: …` — новое, `fix: …` — исправление; ещё `perf`, `update`,
   `delete`, `upstream` (обновление из iBC Shell), `docs`, `refactor`, `build`, `ci`. Коммиты без приставки в
   журнал изменений не попадают и версию не поднимают.
2. После каждого пуша release-please держит открытым PR «chore: подготовить релиз X.Y.Z»: новый номер версии во
   всех файлах и запись в `CHANGELOG.md`. `feat` поднимает среднюю цифру, остальные — последнюю.
3. PR влит — GitHub собирает пять установщиков (macOS Apple Silicon и Intel, Windows, Linux AppImage и .deb),
   прикладывает их к релизу вместе с контрольными суммами и публикует релиз.

## Автообновление

Wave Shell обновляется сам: когда выходит новая версия, внизу боковой панели появляется «Доступна версия…». По
нажатию открывается список изменений и кнопка «Обновить и перезапустить» — приложение скачивает обновление,
ставит его и перезапускается. Работает на macOS, Windows и Linux (AppImage и .deb; для .deb система спросит
пароль администратора).

Обновления подписаны отдельным ключом (это не платная подпись Apple или Microsoft): приложение ставит только
пакеты, подписанные им, поэтому подменённый файл не установится. Открытая часть ключа — в
`src-tauri/tauri.conf.json` (`plugins.updater.pubkey`), закрытая и пароль к ней — в секретах репозитория
**Settings → Secrets and variables → Actions**: `TAURI_SIGNING_PRIVATE_KEY` и `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
При каждом выпуске GitHub подписывает пакеты и кладёт в релиз `latest.json` — его и читает приложение.

Закрытый ключ нельзя терять: без него уже установленные копии не смогут обновиться сами, только переустановкой.

## Основа и лицензия

Wave Shell основан на [iBC Shell](https://github.com/dramaformama/ibc-shell) (автор — Georg, лицензия MIT) и
регулярно получает его обновления. Лицензия — MIT, см. [LICENSE](LICENSE).
