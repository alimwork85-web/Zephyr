# Zephyr Weather Tray

**The weather, one hover away.** Zephyr is a tiny Windows system tray app that shows the current temperature right in your tray icon and the full picture (now, today, tomorrow, rain chance) when you point at it. No browser, no window, no account.

Built in Rust. About 2 MB, around 20 MB of RAM, and free forever (MIT).

## Features

- **Temperature in the icon.** The tray icon itself shows the current temperature, colored from cold blue to hot red, and turns into a white umbrella when rain is likely.
- **Hover for details.** Now, today's high/low, tomorrow's high/low, and an umbrella with the rain chance when it is 30% or more.
- **Any city in the world.** Right-click, choose *Change City...*, type a name and pick from the results. Your choice is remembered.
- **Celsius or Fahrenheit.** One click in the menu.
- **Rain alerts.** One Windows notification when rain starts within the next hour, with the time it begins. Turn it off from the menu.
- **Air quality alerts.** One notification a day when the air is unhealthy (AQI 101 or worse), with the category. Turn it off from the menu.
- **Severe weather alerts.** Official US National Weather Service warnings (severe or extreme) for your city, each one notified once. United States only; elsewhere this is skipped silently.
- **Start with Windows.** Optional, from the menu. No admin rights needed.
- **Always fresh.** Updates every 30 minutes, retries quickly (5 seconds, then longer) if the connection is not ready or drops, and says so in the tooltip instead of showing wrong numbers.
- **Private.** No accounts, no tracking, no analytics. The only network calls are to the free [Open-Meteo](https://open-meteo.com/) forecast and air-quality APIs and to [api.weather.gov](https://api.weather.gov/) for US warnings.

## Download

1. Open the [latest release](https://github.com/alimwork85-web/Zephyr/releases/latest).
2. Download `Zephyr-v0.3.1-windows-x64.exe` and run it. There is no installer.
3. Look for the new icon in the notification area (click the `^` arrow if Windows hides it, and drag the icon out to keep it visible).

> **SmartScreen note:** the EXE is not code-signed yet, so Windows may show "Windows protected your PC". Click *More info*, then *Run anyway*. You can compare the file against the SHA256 checksum published in the release, or build it yourself from source (below).

Requires Windows 10 or 11 (64-bit) and an internet connection.

Tip: keep the EXE in a permanent folder (for example `%LOCALAPPDATA%\Zephyr`) before enabling *Start with Windows*. If you later move or update the file, just run it once and the startup entry fixes itself.

## Usage

| Action | How |
| --- | --- |
| See the forecast | Hover over the tray icon |
| Refresh now | Right-click, *Refresh Now* |
| Change city | Right-click, *Change City...*, search, select, *Use this city* |
| Switch units | Right-click, *Use Celsius* |
| Rain notifications | Right-click, *Rain alerts* (rain starting within the hour) |
| Air quality notifications | Right-click, *Air quality alerts* |
| Severe weather notifications | Right-click, *Severe weather alerts* (US only) |
| Launch at sign-in | Right-click, *Start with Windows* |
| Quit | Right-click, *Exit* |

Settings are stored in `%APPDATA%\Zephyr\config.toml`.

## Build from source

You need the [Rust toolchain](https://rustup.rs/) on Windows (MSVC).

```bash
git clone https://github.com/alimwork85-web/Zephyr.git
cd Zephyr
cargo test
cargo build --release
```

The binary is `target/release/zephyr.exe`.

## How it works

Rust, [tokio](https://tokio.rs/) and [reqwest](https://github.com/seanmonstar/reqwest) for networking, [tray-icon](https://github.com/tauri-apps/tray-icon) and [winit](https://github.com/rust-windowing/winit) for the tray, [native-windows-gui](https://github.com/gabdube/native-windows-gui) for the city picker, and [Open-Meteo](https://open-meteo.com/) for data. The temperature icon is drawn pixel by pixel in code, so there is no font or image dependency.

## Contributing

Issues and pull requests are welcome. Ideas: more languages, hourly view, wind and humidity, code signing.

## License

[MIT](LICENSE). Developed by Ali Alhumedi.

Weather data by [Open-Meteo.com](https://open-meteo.com/) (CC BY 4.0).

---

<div dir="rtl">

# زيفير — الطقس في صينية النظام

**الطقس على بعد تمريرة واحدة.** زيفير تطبيق صغير في صينية نظام ويندوز: درجة الحرارة الحالية داخل الأيقونة، وتفاصيل اليوم وغدًا واحتمال المطر عند تمرير المؤشر. بلا نافذة، بلا حساب، بلا متصفح.

مكتوب بلغة Rust، حجمه نحو 2 ميغابايت، ويستهلك نحو 20 ميغابايت من الذاكرة، ومجاني بترخيص MIT.

## الميزات

- **درجة الحرارة داخل الأيقونة.** بلون من الأزرق (بارد) إلى الأحمر (حار)، وتتحوّل إلى مظلة بيضاء عند توقّع المطر.
- **تمرير المؤشر للتفاصيل.** الآن، العظمى والصغرى لليوم وللغد، ومظلة مع نسبة المطر عندما تبلغ 30% أو أكثر.
- **أي مدينة في العالم.** زر الفأرة الأيمن، *Change City...*، اكتب الاسم واختر من النتائج. يُحفظ اختيارك.
- **مئوية أو فهرنهايت.** نقرة واحدة في القائمة.
- **تنبيه المطر.** إشعار واحد عندما يبدأ المطر خلال الساعة القادمة، مع وقت بدايته. يُطفأ من القائمة.
- **تنبيه جودة الهواء.** إشعار واحد يوميًا عندما يبلغ مؤشر AQI الحد 101 أو أكثر، مع تصنيفه. يُطفأ من القائمة.
- **تحذيرات الطقس الخطيرة.** تحذيرات هيئة الأرصاد الأمريكية (NWS) من مستوى Severe أو Extreme لمدينتك، وكل تحذير يُنبَّه به مرة واحدة. للولايات المتحدة فقط، ويُتجاهل بصمت في غيرها.
- **التشغيل مع ويندوز.** اختياري من القائمة، بلا صلاحيات إدارية.
- **تحديث دائم.** كل 30 دقيقة، وإعادة محاولة سريعة (5 ثوانٍ ثم أطول) إذا لم تجهز الشبكة أو انقطعت، مع كلمة «Offline» في التلميح بدل أرقام خاطئة.
- **خصوصية.** لا حسابات ولا تتبّع ولا إحصاءات. الاتصالات الوحيدة إلى Open-Meteo للطقس وجودة الهواء، وإلى api.weather.gov لتحذيرات أمريكا.

## التنزيل

1. افتح [أحدث إصدار](https://github.com/alimwork85-web/Zephyr/releases/latest).
2. نزّل `Zephyr-v0.3.1-windows-x64.exe` وشغّله. لا يوجد مُثبِّت.
3. ابحث عن الأيقونة في منطقة الإشعارات (اضغط السهم `^` إن أخفاها ويندوز، واسحب الأيقونة لإبقائها ظاهرة).

> **ملاحظة SmartScreen:** الملف غير موقّع رقميًا بعد، وقد يعرض ويندوز «Windows protected your PC». اضغط *More info* ثم *Run anyway*. يمكنك مقارنة بصمة SHA256 المنشورة في الإصدار، أو بناء الملف بنفسك من المصدر.

يتطلب ويندوز 10 أو 11 (64 بت) واتصالًا بالإنترنت.

نصيحة: ضع الملف في مجلد ثابت (مثل `%LOCALAPPDATA%\Zephyr`) قبل تفعيل *Start with Windows*. وإن نقلت الملف أو حدّثته، شغّله مرة واحدة ليصحّح مسار الإقلاع نفسه.

## الاستخدام

| الإجراء | الطريقة |
| --- | --- |
| رؤية التوقعات | مرّر المؤشر على أيقونة الصينية |
| تحديث فوري | زر الفأرة الأيمن، *Refresh Now* |
| تغيير المدينة | *Change City...* ثم ابحث واختر *Use this city* |
| تبديل الوحدة | *Use Celsius* |
| تنبيه المطر | *Rain alerts* (المطر خلال ساعة) |
| تنبيه جودة الهواء | *Air quality alerts* |
| تحذيرات الطقس الخطيرة | *Severe weather alerts* (أمريكا فقط) |
| التشغيل عند تسجيل الدخول | *Start with Windows* |
| الإغلاق | *Exit* |

الإعدادات في `%APPDATA%\Zephyr\config.toml`.

## البناء من المصدر

تحتاج [Rust](https://rustup.rs/) على ويندوز (MSVC).

```bash
git clone https://github.com/alimwork85-web/Zephyr.git
cd Zephyr
cargo test
cargo build --release
```

الملف الناتج: `target/release/zephyr.exe`.

## المطوّر والترخيص

طُوِّر بواسطة **Ali Alhumedi**. الترخيص [MIT](LICENSE)، حقوق النشر (c) 2025-2026 Ali Alhumedi.

بيانات الطقس من [Open-Meteo.com](https://open-meteo.com/) (CC BY 4.0)، والتحذيرات الرسمية من [api.weather.gov](https://api.weather.gov/).

</div>

