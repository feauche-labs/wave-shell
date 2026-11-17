# Changelog

## [1.7.0](https://github.com/feauche-labs/wave-shell/compare/v1.6.1...v1.7.0) (2026-11-17)


### Добавлено

* кнопка обновления в левом меню стала пилюлей «Обновить до …» — с заполнением при скачивании ([96e40df](https://github.com/feauche-labs/wave-shell/commit/96e40df1df8df23f86d3b87bf5df998264e6c362))
* меню как в Arc — кнопки окна в шапке левого меню и прячутся вместе с ним, поиск лупой в верхней панели; пилюля обновления по центру с обводкой ([8d383cc](https://github.com/feauche-labs/wave-shell/commit/8d383cc8ec205b417a215252697d81c4bb3af9a4))
* своё меню вверху экрана на Маке — на языке интерфейса, с настройками, обновлениями, вкладками и масштабом ([6b7cc19](https://github.com/feauche-labs/wave-shell/commit/6b7cc19585fb177bbca0d3082ec850d9c2049b9e))


### Исправлено

* **ci:** меню Мака и кнопки окна больше не ломают проверку сборки на Linux и Windows ([ddee31f](https://github.com/feauche-labs/wave-shell/commit/ddee31feaefb07aa049a879889a8447f69f03047))

## [1.6.1](https://github.com/feauche-labs/wave-shell/compare/v1.6.0...v1.6.1) (2026-11-17)


### Производительность

* клавиши уходят на сервер сразу, без ожидания подтверждения предыдущей — при быстром наборе на 50–100 мс быстрее ([01f2275](https://github.com/feauche-labs/wave-shell/commit/01f2275195c8e7fee04dd64874f032122e43108d))

## [1.6.0](https://github.com/feauche-labs/wave-shell/compare/v1.5.0...v1.6.0) (2026-11-17)


### Добавлено

* масштаб интерфейса в основных настройках — 90–150 %, на Mac ⌘⇧+ / ⌘⇧− / ⌘⇧0 ([ae620bc](https://github.com/feauche-labs/wave-shell/commit/ae620bcdd5b3dc0e84abeb4cd0d645ca2a50916f))
* цвет папки заливает её строку, а раскрытую папку — целиком вместе с хостами ([f7cf02c](https://github.com/feauche-labs/wave-shell/commit/f7cf02cb309ddad936ceeec9457a845f4681c2cf))


### Исправлено

* «Переподключиться» больше не зависает на «Подключение» — отказ возвращает вкладку в «Отключено» ([fafb197](https://github.com/feauche-labs/wave-shell/commit/fafb1979a8cad39110d6864380aa9755fe2541bb))
* «продолжить для всех» больше не затирает другой файл с тем же именем — спрашивает про него отдельно ([75cc443](https://github.com/feauche-labs/wave-shell/commit/75cc443ca7995ecd3558b92973ea6a2fc1b32215))
* два файла из одной очереди больше не записываются в один (одинаковые имена, Readme и README) ([5f251fd](https://github.com/feauche-labs/wave-shell/commit/5f251fd2d90cbf83b337fb9495a3a51ed41b81a8))
* если докачать файл не удалось, он больше не пропадает из папки ([3033a44](https://github.com/feauche-labs/wave-shell/commit/3033a44cedcff836ae78a6a58ab27bcc6f0a5d92))
* новый тип ключа у сервера больше не даёт ложную блокировку «ключ изменился» — сначала просим уже сохранённый тип ([5166486](https://github.com/feauche-labs/wave-shell/commit/51664866cfcdf845d0461534af607979091f00ca))
* папка со значком или смайликом снова сохраняется — расширено ограничение длины в базе ([f1bc1fa](https://github.com/feauche-labs/wave-shell/commit/f1bc1fa95189710abc66018ae9bdfdcff6edbf69))
* подсветка в терминале не замолкает после вставки текста — цвета отслеживаются по отдельности ([6c0c1e7](https://github.com/feauche-labs/wave-shell/commit/6c0c1e7485f405597e6992b6dae83db2e23fc341))
* после закрытия вкладки «Файлы» её передачи честно отменяются, а не висят; «Повторить» объясняет, почему не может ([7bd3944](https://github.com/feauche-labs/wave-shell/commit/7bd3944280eb809969244de17a6486bb4e50e58f))
* терминал не зависает при большом выводе — ввод уходит отдельно, а отстающий экран не тормозит «Состояние» и «Файлы» ([9f4ee8e](https://github.com/feauche-labs/wave-shell/commit/9f4ee8e42cf705356996c8325f184de5f31ea5e0))


### Производительность

* нажатия клавиш уходят на сервер сразу, а не на следующем кадре экрана — ввод быстрее и ровнее ([765f5db](https://github.com/feauche-labs/wave-shell/commit/765f5db59ebd0122a3612993ecd4724e24ecad02))


### Тесты

* тесты больше не зависят от локальных ключей и прав доступа Unix — CI на Linux и Windows снова проходит ([496d084](https://github.com/feauche-labs/wave-shell/commit/496d0843bb6f02ba0e563b8ebff3354af633031b))
* тесты конфликтов файлов и переподключения больше не падают на быстрых и медленных машинах CI ([9dc6474](https://github.com/feauche-labs/wave-shell/commit/9dc64745226a45b9d23f2f4661a710d1eeb9b87f))

## [1.5.0](https://github.com/feauche-labs/wave-shell/compare/v1.4.0...v1.5.0) (2026-11-16)


### Добавлено

* значки и цвета папок — флаг, значок или смайлик с поиском, 10 мягких цветов, цветная линия вдоль хостов папки ([2c3a16b](https://github.com/feauche-labs/wave-shell/commit/2c3a16b84a6128ea2fc223a7f72450b45bda38c0))


### Исправлено

* «применить ко всем» при совпадении имён файлов срабатывает для всех выбранных файлов, остальные больше не зависают ([cfa5048](https://github.com/feauche-labs/wave-shell/commit/cfa504800079cd14e04359803a1a1fa3bfbb95b6))
* автоблокировка больше не срабатывает, пока вы печатаете в терминале ([2e4d456](https://github.com/feauche-labs/wave-shell/commit/2e4d4567cf4f2966bf442e35f60e9416d6856b18))
* в меню папки и хоста снова открываются «Флаг» и «Переместить в» ([cd9992e](https://github.com/feauche-labs/wave-shell/commit/cd9992e7760a8fe409bbb85c5cf1275669268108))
* в разделённом экране буквы уходят в подсвеченное окно, а не в соседнее ([0a87711](https://github.com/feauche-labs/wave-shell/commit/0a87711d4f6907f2f238745ffebb421dd60e8e0a))
* загрузка на сервер, который не даёт менять дату файла, больше не обрывается в конце ([a305fc3](https://github.com/feauche-labs/wave-shell/commit/a305fc32eaf1366862f8504dbe2432c7f7304099))
* подменю «Флаг» и «Переместить в» больше не прилипает к меню ([b927a44](https://github.com/feauche-labs/wave-shell/commit/b927a44bbcefc28b39b1937f78d9c93b5c463ea4))
* терминал больше не прячет приглашение и набираемые буквы в fish, vim и neovim ([fe646ae](https://github.com/feauche-labs/wave-shell/commit/fe646ae304b397d8b689f7120f5f621dda21a8c3))


### Из iBC Shell

* iBC Shell 2.0.2 — возврат хоста из отдельного окна во вкладки (без отрыва вкладки) ([43053f6](https://github.com/feauche-labs/wave-shell/commit/43053f643b0ce6e09d00fb38ec5712109de51c59))

## [1.4.0](https://github.com/feauche-labs/wave-shell/compare/v1.3.0...v1.4.0) (2026-11-16)


### Добавлено

* окна рабочей области — шапка как в Termius, перетаскивание за шапку, ввод во все окна; меню как в Arc; плавные панели; сниппеты — раскрывающиеся папки и раздел плитками ([514d625](https://github.com/feauche-labs/wave-shell/commit/514d625f7967023bcb9988cfc4a264593b1ed1ba))

## [1.3.0](https://github.com/feauche-labs/wave-shell/compare/v1.2.0...v1.3.0) (2026-11-16)


### Добавлено

* адреса и трафик по адресам в «Состоянии», поиск по тегам ([6f3e80f](https://github.com/feauche-labs/wave-shell/commit/6f3e80f6425cf7ca34f032a469ac24603452f1e9))
* разделение экрана как в Termius — перетащите вкладку на терминал, панели с размером, развернуть, вернуть во вкладку; вкладка больше не вылетает в отдельное окно ([69a7a1f](https://github.com/feauche-labs/wave-shell/commit/69a7a1f57bad2b3b8d7ebbcfba3fba339439716c))

## [1.2.0](https://github.com/feauche-labs/wave-shell/compare/v1.1.0...v1.2.0) (2026-11-15)


### Добавлено

* раздел «Ключи» как в Termius — плитки, добавление текстом или файлом, пароль ключа, установка на сервер; в хосте ключ выбирается из списка ([6c005c7](https://github.com/feauche-labs/wave-shell/commit/6c005c7d13217a685662a2f0adaa071b7d20dc0d))
* теги у хостов — плашки с подсказками в окне хоста, теги в «Сводке» и фильтр по тегу ([d8a14e2](https://github.com/feauche-labs/wave-shell/commit/d8a14e265d35700602e72411bf6d9b5a81cfbe66))


### Исправлено

* в «Доменах» больше не показывается системный набор корневых сертификатов (ACCVRAIZ1) ([cbf5b0e](https://github.com/feauche-labs/wave-shell/commit/cbf5b0e08ddf02320961a61870691d6b3bf40b14))
* вкладка больше не вылетает в отдельное окно, если её случайно вытащить за полосу вкладок ([bd84945](https://github.com/feauche-labs/wave-shell/commit/bd84945a4286f2f42f848254aceb9c1068104242))


### Производительность

* эхо в терминале без задержки — вывод после паузы сразу на экран (было +10 мс на каждую клавишу) ([727be30](https://github.com/feauche-labs/wave-shell/commit/727be305133ab928ce7c06a86c41f0e06e840063))

## [1.1.0](https://github.com/feauche-labs/wave-shell/compare/v1.0.0...v1.1.0) (2026-11-15)


### Добавлено

* обновление одной кнопкой — «что нового», загрузка, установка и перезапуск (macOS, Windows, Linux) ([2a2b3a5](https://github.com/feauche-labs/wave-shell/commit/2a2b3a5a1679d40295c638bf7a81511bd8b3b5ec))
* свой значок Wave Shell — стрелка и бирюзовая волна (приложение, строка меню, вкладка) ([26fad0e](https://github.com/feauche-labs/wave-shell/commit/26fad0e7ab5b5c18aa48e74a74df66993501e01e))


### Исправлено

* кнопки в пустом списке хостов — по центру и не вылезают за край панели, импорт ссылкой под кнопкой ([7f82e0f](https://github.com/feauche-labs/wave-shell/commit/7f82e0fc9feba06a4ac2834569823f72ac587db2))


### Изменено

* тема по умолчанию — как в системе ([cabb540](https://github.com/feauche-labs/wave-shell/commit/cabb540243267951be4dd473125fd6cfef92e4e8))

## 1.0.0 (2026-11-15)


### Добавлено

* Wave Shell — своя сборка iBC Shell 2.0.1 ([3f029a0](https://github.com/feauche-labs/wave-shell/commit/3f029a03e5e7e1d5e04b8aa360ad25266386683a))
