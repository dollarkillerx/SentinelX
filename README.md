# 🎯 产品定位

这是一个 **去中心化的流量转发与管理系统**，客户端（Client）可通过 Server 下发的任务进行自动化控制，支持 **iptables 管理、端口转发、匿名流量中转**。最终目标是：

* 统一管理分布在不同节点上的 Client；
* 提供灵活的流量转发（直连 / 加密 / WebSocket 封装）；
* 支持 **流量统计、限速、转发策略配置**；
* 提供可视化 Web UI 供运维和运营使用。

---

# 📦 功能模块设计

## 1. Client 端

* **1.0 注册与任务接收**

  * 启动时向 Server 注册（身份认证 + 心跳维持）。
  * 接收 Server 下发的任务（iptables 更新、转发规则更新）。

* **1.1 iptables 管理**

  * 支持基本规则下发与执行。
  * 规则可本地生效，也可批量下发（Server 控制）。

* **1.2 端口转发**

  * 默认监听 `0.0.0.0:随机端口`（可自定义）。 监听ip默认0.0.0.0 端口自定义
  * 转发目标可配置（例：`127.0.0.1:8080`）。
  * 支持 **流量统计（字节/连接数）** 与 **限速**。

* **1.3 流量中继**

  * 支持选择一个 Client 作为入口，另一个 Client 作为出口。
  * 转发方式：

    1. TCP 直连
    2. TCP 加密转发（自研简单加密或 TLS）
    3. 封装到 WebSocket + 加密转发（适合绕过限制）
  * 场景：公网 → Client1 → Client2 → 内网服务

---

## 2. Server 端

* **2.1 Client 管理与统计**

  * 显示所有注册 Client 状态（在线/离线、流量使用、规则版本）。
  * 心跳检测 + 自动失效清理。

* **2.2 iptables 控制**

  * 批量下发规则。
  * 可对单个 Client 或 Client 组应用策略。

* **2.3 转发与任务管理**

  * 配置入口/出口 Client 的映射。
  * 下发端口转发策略。
  * 提供流量统计与限速设置。

---

## 3. Web UI

* **3.1 管理面板**

  * Client 列表（状态、流量、运行时信息）。
  * 转发规则配置（入口/出口映射）。
  * iptables 策略管理（批量推送）。

* **3.2 前端技术**

  * React + Radix UI 组件库（简洁、现代）。
  * REST/gRPC 接口调用后端 Server。

---

# ⚙️ 技术架构（CS 架构 + Rust 技术栈）

## 架构图（简化）

```
[ Web UI (React) ]
        |
   [ Server (Rust, actix/axum) ]
        |   \
        |    ---- Client 状态/任务存储 (Postgres)
        |
   [ Client (Rust) ] <----> [ Client (Rust) ]
   - iptables 控制             - 出口转发
   - 端口转发                  - 转发目标
   - 加密/WS 封装
```

## 技术选型

* **语言**: Rust（高性能、低资源占用、适合网络编程 + 安全性高）
* **Server**

  * 框架: `axum` （jsonrpcAPI）
  * 存储: `Postgres`（存 Client 状态 & 规则）
  * 通信: jsonrpc（Client ↔ Server 任务下发）
* **Client**

  * 网络: `tokio` + `tokio-tungstenite`（WebSocket 支持）
  * 系统操作: `iptables` 控制通过 `nftables`/`iptables` CLI 或 `netlink` crate
  * 转发: 自研 TCP Proxy + 可插拔加密模块
  * 限速: `tokio_util::Limiter` 或 Token Bucket 实现
* **Web UI**

  * React + Radix UI + Tailwind
  * 调用 Server 提供的 API

---

# 🚀 交付阶段（MVP → 完整版）

1. **MVP 阶段**

   * Client 注册 + 心跳
   * Server 基础管理面板（列表、状态）
   * Client 支持端口转发（TCP 直连）
   * 简单流量统计（字节数）

2. **进阶版**

   * 支持 iptables 控制
   * 支持加密转发（TCP + WS）
   * 限速功能
   * Web UI 全量功能上线

3. **完善版**

   * 分组管理（Client 分组推送规则）
   * 流量报表（图表、导出）
   * 权限管理（多用户 Web UI 登录）
   * HA（Server 多实例容灾）
