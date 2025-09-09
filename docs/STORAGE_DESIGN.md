# 后端存储设计与使用说明

## 目标

监听链上 ShowManager 合约 `ShowCreated` 等事件，落库到 PostgreSQL，提供后续查询/聚合能力，并保证幂等（重复日志不产生重复记录）。

## 总体架构

```
WebSocket Provider (alloy) ---> 事件路由 router.rs ---> 事件解析 show_manager.rs
                                                    |-> 原始日志表 show_created_events
                                                    |-> 结构化表 show_created_events_detail (幂等 upsert)
```

## 模块说明

- `src/db/mod.rs`: 提供 `Db` 封装，创建 `PgPool`，留有迁移钩子。
- `src/event/router.rs`: 根据合约地址路由日志到相应解析器。
- `src/event/show_manager.rs`: 解码 `ShowCreated`，写入两张表。
- `src/repo/show_repo.rs`: 定义结构化记录 `ShowCreatedRecord` 及 `insert_show_created` upsert 逻辑。
- `migrations/0001_init.sql`: 建表 SQL（原始+结构化）。

## 数据表设计

1. `show_created_events`
   - 保存原始日志及解码后的 JSON 片段 `raw_event`。
   - 唯一键: `(tx_hash, log_index)` 防止重复。
   - 字段: `tx_hash, block_number, contract_address, log_index, raw_event(JSONB), created_at`。
2. `show_created_events_detail`
   - 保存结构化字段：`show_id, organizer, name, start_time, end_time, venue`，以及链位置信息。
   - 主键: `show_id`，重复事件更新最新数据（幂等 / 可覆盖）。

## 事件字段映射 (ShowCreated)

| Solidity                    | Rust decoding   | DB detail 列       |
| --------------------------- | --------------- | ------------------ |
| showId (uint256 indexed)    | event.showId    | show_id            |
| organizer (address indexed) | event.organizer | organizer (0x…hex) |
| name (string)               | event.name      | name               |
| startTime (uint256)         | event.startTime | start_time         |
| endTime (uint256)           | event.endTime   | end_time           |
| venue (string)              | event.venue     | venue              |

U256 -> i64: 通过 `try_into()` 取低 64 位；假设不会超过 `i64::MAX`。可在后续加入范围检查与告警。

## 关键代码片段概览

> 仅示意，实际代码请查看对应文件。

- 连接数据库: `Db::connect(DATABASE_URL, 5)`
- 事件路由: `route_log(log, &addr_map, &flags, &db)`
- 写库（原始表）: `INSERT ... ON CONFLICT DO NOTHING`
- Upsert 详情: `ON CONFLICT (show_id) DO UPDATE SET ...`

## 环境变量

```
DATABASE_URL=postgres://user:password@localhost:5432/ticket
DID_REGISTRY_ADDRESS=0x...
SHOW_MANAGER_ADDRESS=0x...
PRINT_RAW_LOGS=1          # 可选
PRINT_UNKNOWN_LOGS=1      # 可选
```

## 初始化步骤

1. 创建数据库：`createdb ticket` 或手动。
2. 执行迁移：
   ```bash
   psql "$DATABASE_URL" -f backend/migrations/0001_init.sql
   ```
   （或集成 sqlx migrate: 后续可添加 `sqlx::migrate!("migrations")`。）
3. 运行本地链（Foundry / Anvil）并部署合约，写入 `.env` 中的合约地址。
4. 启动后端：
   ```bash
   cargo run
   ```
5. 触发 `createShow` 事务后检查数据库：
   ```sql
   SELECT * FROM show_created_events_detail ORDER BY created_at DESC LIMIT 5;
   ```

## 扩展规划

| 方向              | 说明                                                                     |
| ----------------- | ------------------------------------------------------------------------ |
| 追加更多事件      | ShowUpdated / ShowCancelled 等，新增对应 repository 与表。               |
| 查询 API          | 在 `listen_app` 的 `axum::Router` 中暴露 REST: GET /shows, /shows/{id}。 |
| 分页与过滤        | 针对 organizer / 时间范围加复合索引。                                    |
| Migrations 自动化 | 使用 `sqlx migrate add` + `sqlx::migrate!`。                             |
| 数据质量监控      | 记录解析失败日志表。                                                     |
| Range 校验        | 对 U256 -> i64 超界值告警并落入 dead-letter 表。                         |

## 常见问题 (FAQ)

1. 构建时报错 `set DATABASE_URL`：使用 `sqlx::query` (已处理) 或设置编译期环境并执行 `cargo sqlx prepare`。
2. 出现重复事件：通过唯一键或主键 upsert 幂等化。若链回滚，可考虑加入 `block_hash` 与 `removed` 标记处理 (alloy log 包含 `removed`) —— 后续可扩展。
3. 性能考虑：当前同步写入；高吞吐可引入 `mpsc` 队列 + 批量写入 + 事务。

## 下一步建议

- 添加 ShowUpdated 等事件解析。
- 增加简单查询接口（示例：GET /shows?organizer=0x...）。
- 引入 tracing + structured logging。

---

设置环境变量:
创建数据库并运行迁移（当前简单 SQL 文件）:
或用 sqlx-cli（可选）:

同时需要 .env 添加:
运行本地链（确保 ws://127.0.0.1:8545 可达），再启动:
可扩展建议（后续可做）
使用 sqlx::migrate! 宏正式管理迁移。
增加其它事件 (ShowUpdated/ShowCancelled/ShowActivated/ShowEnded) 表与仓储。
为 U256 转换添加范围校验与错误日志。
在 axum Router 中添加查询接口（例如 GET /shows）。
需要我继续添加其它事件的存储或提供 REST 查询接口请直接说明。
