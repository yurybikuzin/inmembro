# Тестовое задание: in-memory брокер сообщений

## Задание: 

разработать и реализовать in-memory брокер сообщений на языке Rust. 

Брокер должен обеспечивать распределение потоков сообщений по топикам, при этом каждое сообщение может иметь опциональный ключ. 

Формат сообщения и ключа, а также протокол взаимодействия с брокером выбирает кандидат (например, байтовый массив или JSON для данных; TCP или HTTP + SSE для взаимодействия).

## Требуемые функции
Брокер должен поддерживать следующие операции:
- Создание топика: добавление нового топика для сообщений.
- Подписка на сообщения: регистрация клиента на получение сообщений из указанного топика.
- Отписка от сообщений: прекращение отправки сообщений клиенту из указанного топика.
- Добавление сообщения: отправка сообщения в конкретный топик для дальнейшей доставки подписчикам.

## Дополнительные функции (будет плюсом)

Реализация следующих функций будет преимуществом:
- Retention для топиков: поддержка параметра времени хранения сообщений в топике, по истечении которого сообщение удаляется и становится недоступным для подписчиков.
- Compaction для топиков: поддержка механизма компакции сообщений. При этом в топике сохраняется только последнее сообщение с определённым ключом, все предыдущие сообщения с таким же ключом удаляются. См. документацию Kafka для справки.
- Commit-механизм: сообщение считается доставленным, только если клиент явно сообщил об этом брокеру. До подтверждения доставки сообщение должно оставаться доступным для повторной отправки.

## Требования к реализации:
- Язык: Rust.
- Использование библиотек/фреймворков остаётся на усмотрение кандидата.

# Среда разработки

- ОС: Ubuntu 22.04.5 LTS
- Rust: rustc 1.82.0 (f6e511eec 2024-10-15)

# Что сделано

## Реализован веб-сервер (на базе фреймворка [axum](https://github.com/tokio-rs/axum)), поддерживающей следующие endpoint'ы:

- `GET /topic/:name/create` - Создание топика: добавление нового топика для сообщений
- `POST /topic/:name/push` - Добавление сообщения: отправка сообщения в конкретный топик для дальнейшей доставки подписчикам. Payload: `{ "key": Option<String>, "data": serde_json::Value }`
- `GET /topic/:name/push` - GET-версия добавления сообщения. Для целей упрощенной отладки (использование в адресной строке браузера). Соообщение передается через query-параметр `message`
- `GET /topic/:name/subscribe` - Подписка на сообщения: регистрация клиента на получение сообщений из указанного топика. В результате запроса формируется поток [Server Sent Events](https://developer.mozilla.org/ru/docs/Web/API/Server-sent_events). Отписка от сообщений происходит автоматически в результате закрытия потока (страницы в браузере).

- `GET /` - для целей отладки - "индексная страница" - выводит состояние внутренних структур сервера. 
- `GET /about` - для сервисных целей (например, мониторинга) - about-страница веб-сервера

### Намеченные, но НЕ реализованные (по существу) endpoint'ы:

- `POST /topic/:name/config` - Позволяет *опционально* задавать параметры topic'а (**retention**/**compaction**). Payload: `{ "retention": Option<bool>, "compaction": Option<bool> }`. Возвращает `application/json` с актуальными (установленными) значениями параметров: `{ "retention": bool, "compaction": bool }`

- `GET /topic/:name/config` - для целей-отладки - страница, отражающая настройки topic'а (включен ли **retention**/**compaction**?). Этот же endpoint при указании соответствующих query-параметров позволяет задавать новые значения этих настроек.

## Концепция решения 

Все топики хранятся в поле `topics: HashMap<String, Arc<std::sync::RwLock<Topic>>>` shared-структуры `struct AppState` фреймворка `axum`, передаваемой в handler каждого endpoint'а в качестве параметра `state: Extension<SharedState>`, где `type SharedState = Arc<tokio::sync::RwLock<AppState>>`

Каждый топик имеет свой `TopicContent` и `TopicConfig`:
```
struct Topic {
    pub content: TopicContent,
    pub config: Arc<TopicConfig>,
}
```

При этом `TopicConfig` содержит значения параметров топика: `retention_millis`, `compaction`:
```
struct TopicConfig {
    /// включен ли режим **retention** ?
    /// и если да (Some), то время жизни сообщения в миллисекундах
    retention_millis: Option<u64>,

    /// включен ли режим **compaction** ?
    /// по умолчанию (None) считается отключенным
    compaction: Option<bool>,
}
```

`TopicContent` в зависомости от наличия/отсутствия подписчиков может быть в одном из двух состояний:
```
enum TopicContent {
    /// нет ни одного подписчика, и сообщения копятся в одной очереди сообщений топика
    Messages(Messages), 

    /// для каждого подписчика формируется своя очередь сообщений, поскольку 
    /// у каждого подписчика может быть своя скорость потребления сообщений; 
    /// но каждое новое поступившее сообщение помещается в очередь каждого из подписчиков
    Subscribers(Vec<Weak<Subscriber>>), 
}

struct Subscriber {
    /// для возможности идентификации подписчиков, 
    /// что необходимо для реализации автоматической отписки от сообщений
    id: uuid::Uuid, 

    /// собственная очередь сообщений подписчика, 
    /// поскольку у него может быть собственная скорость потребления сообщений
    messages: Arc<std::sync::RwLock<Messages>>,
    ...
}

struct Messages(VecDeque<Arc<QueuedMessage>>);

struct QueuedMessage {
    content: Message,

    /// для поддержки Retention
    created_at: std::time::Instant, 
}

struct Message {
    data: Option<serde_json::Value>,

    /// для поддержки Compaction
    key: Option<String>, 
}
```

### Создание топика - `GET /topic/:name/create`

При создании топика ( `async fn create_topic` ) проверяем его существование (`Entry::Occupied(_) => format!("topic'{name}' already exists")`), и если это новый топик, то создаем его вариант *по умолчанию*:
```
Entry::Vacant(e) => {
    e.insert(Arc::new(std::sync::RwLock::new(Topic::default())));
    format!("created topic'{name}'")
}
```
при этом для `Topic::default()` с учетом 
```
struct Topic {
    pub content: TopicContent,
    pub config: Arc<TopicConfig>,
}
enum TopicContent {
    Messages(Messages),
    Subscribers(Vec<Weak<Subscriber>>),
}
```
предполагается
```
impl Default for TopicContent {
    fn default() -> Self {
        Self::Messages(Messages::default())
    }
}
```
при этом 
```
#[derive(Default)]
struct TopicConfig {
    retention_millis: Option<u64>,
    compaction: Option<bool>,
}
```

### Подписка на топик - `GET /topic/:name/subscribe`

При подписке на топик можно в качестве query-параметров задать 
```
struct SubscribeParams {
    /// Настройка для SSE
    keep_alive_secs: Option<u64>,

    /// Настройка для SSE
    keep_alive_text: Option<String>,

    /// Создавать ли топик при подписке на еще несуществующий топик?
    anyway: Option<bool>,
}
```

При подписке на *существующий* топик (который при необходимости - **anyway**? - создается) формируется `EventStream` (`let event_stream = EventStream::new(&topic);`): 
```
struct EventStream(Arc<Subscriber>);

impl EventStream {
    pub fn new(topic: &Arc<std::sync::RwLock<Topic>>) -> Self {
        let topic_config = Arc::downgrade(&topic.read().unwrap().config);
        let topic = Arc::downgrade(topic);
        Self(Arc::new(Subscriber {
            topic,
            topic_config,
            messages: Arc::default(),
            id: uuid::Uuid::new_v4(),
        }))
    }
    pub fn subscriber(&self) -> Weak<Subscriber> {
        Arc::downgrade(&self.0)
    }
}
```
что приводит к появлению `Subscriber`'а, который добавляется в `TopicContent`:
```
enum TopicContent {
    Messages(Messages),
    Subscribers(Vec<Weak<Subscriber>>),
}
```
Поскольку наблюдается картина перекрестного использования структур друг другом, то, для того, чтобы избежать ситуации [reference cycle](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html), используется [`struct std::sync::Weak`](https://doc.rust-lang.org/beta/std/sync/struct.Weak.html).

Это позволяет корректно реализовать автоматическую *отписку от сообщений* при закрытии потока SSE:

```
impl Drop for EventStream {
    fn drop(&mut self) {
        debug!("EventStream::drop: {:?}", self.0.id());
        if let Some(topic) = self.0.topic.upgrade() {
            debug!("EventStream::drop: will remove_subscriber");
            topic.write().unwrap().remove_subscriber(self.subscriber());
        }
    }
}
```

## Проверка решения

### Запуск веб-сервера

В папке `src/rust/inmembro`:
```
make server
```

Примечание: `Makefile` используется лишь для сокращения часто выполняемых команд (в данном случае, `cargo run --release -- server`)  вкупе с "бесплатной поддержкой" *bash completion of Makefile target*

### Создание топика `some`

В браузере открыть страницу [http://localhost:42084/topic/some/create](http://localhost:42084/topic/some/create)

### Отправка сообщения в топик `some`

В браузере открыть страницу [http://localhost:42084/topic/some/push?message={"data":{"some":"content"},"key":"some_key"}](http://localhost:42084/topic/some/push?message={"data":{"some":"content"},"key":"some_key"})

### Проверка состояния внутренних структур веб-сервера

В браузере открыть страницу [http://localhost:42084/](http://localhost:42084/)

### Подписка на сообщения топика `some`

В браузере открыть страницу [http://localhost:42084/topic/some/subscribe](http://localhost:42084/topic/some/subscribe)

### Далее можно:
- открыть еще одну страницу с подпиской на сообщения топика `some`
- проверять состояние внутренних структур веб-сервера
- отправлять дополнительные сообщения в топик `some`
- закрыть страницу с подпиской на сообщения в топике; наблюдать, как это отражается на состоянии внутренних структур веб-сервера (необходимо обновить ранее открытую страницу)
- проверить другие работу с другими топиками

# Недостатки текущего состояния реализации

- не все endpoint'ы реализованы (речь идет о `GET/POST /topic/:name/config`)
- в endpoint'е `GET /topic/:name/create` было бы полезно добавить поддержку query-параметров, позволяющих задать начальные значения конфигурации топика:
```
struct TopicConfig {
    /// включен ли режим **retention** ?
    /// и если да (Some), то время жизни сообщения в миллисекундах
    retention_millis: Option<u64>,

    /// включен ли режим **compaction** ?
    /// по умолчанию (None) считается отключенным
    compaction: Option<bool>,
}
```
- не реализован вариант взамодействия с брокером по TCP с использованием кодирования структуры в байтовый массив используя [protobuf](https://github.com/stepancheg/rust-protobuf)/[bincode](https://docs.rs/bincode/latest/bincode/)/[Message pack](https://docs.rs/rmp/latest/rmp/) and so on, в зависомости от требований клиентов, использующих брокер собщений
- не реализована поддержка commit-механизма, уместного при взаимодействии с брокером по TCP
- не реализованы [интеграционные тесты](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
- код недокументирован, соответственнно команда `cargo doc --open` не приводит к содержательному результату

