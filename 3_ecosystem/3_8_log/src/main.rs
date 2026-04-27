use std::fs::File;
use std::io;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    prelude::*,
    Layer,
    filter::LevelFilter,
};
use time::macros::format_description;

fn main() {
    // 1. Настройка формата времени (RFC 3339 с наносекундами)
    // Т.к. стандартный формат в tracing-subscriber может быть ограничен,
    // используем кастомный из библиотеки `time`.
    let timer = UtcTime::new(format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]Z"
    ));

    // 2. Слой для app.log -> STDOUT (только INFO и DEBUG, исключая WARN/ERROR)
    // Для простоты примера сделаем разделение: STDOUT получает всё ниже WARN
    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        .json()
        .with_timer(timer.clone())
        .with_filter(LevelFilter::INFO)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.level() > &Level::WARN // В tracing чем больше число, тем ниже приоритет
        }));

    // 3. Слой для app.log -> STDERR (WARN и выше)
    let stderr_layer = fmt::layer()
        .with_writer(io::stderr)
        .json()
        .with_timer(timer.clone())
        .with_filter(LevelFilter::WARN);

    // 4. Слой для access.log (в файл)
    // Чтобы писать только логи с определенным полем или именем, используем фильтрацию по target
    let file = File::create("access.log").expect("Не удалось создать файл");
    let access_log_layer = fmt::layer()
        .with_writer(file)
        .json()
        .with_timer(timer)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target() == "access_log"
        }));

    // 5. Инициализация глобального подписчика
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(stderr_layer)
        .with(access_log_layer)
        .init();

    // --- Примеры использования ---

    // Обычные логи приложения
    info!(file = "app.log", "App started");
    warn!(file = "app.log", "Something looks fishy");
    error!(file = "app.log", "Error occurred");

    // Логи доступа (используем target, чтобы направить в нужный слой)
    info!(
        target: "access_log",
        file = "access.log",
        method = "POST",
        path = "/some",
        "http"
    );
}