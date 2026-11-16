# Changelog

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
