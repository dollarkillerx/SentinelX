use anyhow::Result;
use jsonrpsee::server::RpcModule;
use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use sentinel_common::{
    ClientInfo, HeartbeatRequest, HeartbeatResponse, MetricsSummary, RegisterRequest,
    RegisterResponse, SystemMetrics, Task, RelayConfig, TaskType, IptablesRule,
};
use std::sync::Arc;

use crate::manager::ClientManager;

pub async fn create_rpc_module(manager: Arc<ClientManager>) -> Result<RpcModule<Arc<ClientManager>>> {
    let mut module = RpcModule::new(manager);

    module.register_async_method("client.register", |params, ctx, _| async move {
        let req: RegisterRequest = params.parse()?;
        let token = ctx.register_client(req.client_info).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<RegisterResponse, ErrorObjectOwned>(RegisterResponse {
            client_id: "".to_string(),
            token,
        })
    })?;

    module.register_async_method("client.heartbeat", |params, ctx, _| async move {
        let req: HeartbeatRequest = params.parse()?;

        ctx.update_heartbeat(&req.client_id).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        if let Some(metrics) = req.metrics {
            ctx.update_metrics(&req.client_id, metrics).await
                .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;
        }

        let tasks = ctx.get_pending_tasks(&req.client_id).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<HeartbeatResponse, ErrorObjectOwned>(HeartbeatResponse {
            status: "ok".to_string(),
            tasks,
        })
    })?;

    module.register_async_method("client.list", |_, ctx, _| async move {
        Ok::<Vec<sentinel_common::ClientInfo>, ErrorObjectOwned>(ctx.list_clients().await)
    })?;

    module.register_async_method("metrics.get_summary", |_, ctx, _| async move {
        let clients = ctx.list_clients().await;

        Ok::<MetricsSummary, ErrorObjectOwned>(MetricsSummary {
            total_clients: clients.len() as u32,
            online_clients: clients.len() as u32,
            total_cpu_usage: 0.0,
            total_memory_usage: 0.0,
            total_bandwidth_rx: 0,
            total_bandwidth_tx: 0,
        })
    })?;

    module.register_async_method("relay.start", |params, ctx, _| async move {
        #[derive(serde::Deserialize)]
        struct StartRelayRequest {
            entry_client_id: String,
            exit_client_id: String,
            entry_point: String,
            exit_point: String,
            transport_type: sentinel_common::TransportType,
        }

        let req: StartRelayRequest = params.parse()?;

        let relay_config = RelayConfig {
            entry_point: req.entry_point,
            exit_point: req.exit_point,
            transport_type: req.transport_type,
        };

        ctx.create_relay_task(&req.entry_client_id, relay_config).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({"status": "relay_started"}))
    })?;

    module.register_async_method("relay.stop", |params, ctx, _| async move {
        #[derive(serde::Deserialize)]
        struct StopRelayRequest {
            client_id: String,
            entry_point: String,
            exit_point: String,
        }

        let req: StopRelayRequest = params.parse()?;

        let relay_config = RelayConfig {
            entry_point: req.entry_point,
            exit_point: req.exit_point,
            transport_type: sentinel_common::TransportType::Direct, // Doesn't matter for stop
        };

        ctx.create_stop_relay_task(&req.client_id, relay_config).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({"status": "relay_stopped"}))
    })?;

    module.register_async_method("iptables.update", |params, ctx, _| async move {
        #[derive(serde::Deserialize)]
        struct UpdateIptablesRequest {
            client_id: String,
            rules: Vec<IptablesRule>,
        }

        let req: UpdateIptablesRequest = params.parse()?;

        ctx.create_iptables_task(&req.client_id, req.rules).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({"status": "iptables_task_created"}))
    })?;

    module.register_async_method("iptables.apply_rule", |params, ctx, _| async move {
        #[derive(serde::Deserialize)]
        struct ApplyRuleRequest {
            client_id: String,
            rule: IptablesRule,
        }

        let req: ApplyRuleRequest = params.parse()?;

        ctx.create_iptables_task(&req.client_id, vec![req.rule]).await
            .map_err(|e| ErrorObjectOwned::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>))?;

        Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({"status": "iptables_rule_queued"}))
    })?;

    Ok(module)
}