use base64::Engine;
use prost::Message;
use serde_json::Value;

/// 将 jetskiStateSync.agentManagerInitState 作为 SessionResponse proto 解码
pub fn decode_jetski_state_proto(b64: &str) -> Result<Value, String> {
    if b64.trim().is_empty() {
        return Err("jetskiStateSync.agentManagerInitState 为空".to_string());
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| {
            format!(
                "jetskiStateSync.agentManagerInitState Base64 解码失败(len={}): {}",
                b64.len(),
                e
            )
        })?;

    let msg = crate::proto::SessionResponse::decode(bytes.as_slice()).map_err(|e| {
        format!(
            "jetskiStateSync.agentManagerInitState Protobuf 解码失败(len={}): {}",
            bytes.len(),
            e
        )
    })?;

    Ok(session_response_to_json(&msg))
}

fn session_response_to_json(msg: &crate::proto::SessionResponse) -> Value {
    let b64 = |data: &Vec<u8>| {
        if data.is_empty() {
            None
        } else {
            Some(base64::engine::general_purpose::STANDARD.encode(data))
        }
    };

    // 认证信息
    let auth = msg.auth.as_ref().map(|a| {
        serde_json::json!({
            "access_token": a.access_token,
            "token_type": a.token_type,
            "refresh_token": a.refresh_token,
            "created_at": a.created_at.as_ref().map(|t| t.seconds),
        })
    });

    // 模型配置
    let models = msg
        .context
        .as_ref()
        .and_then(|ctx| ctx.models.as_ref())
        .map(|m| {
            let items: Vec<Value> = m
                .items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "name": item.name,
                        "id": item.id.as_ref().map(|id| id.id),
                        "field_5": item.field_5,
                        "field_11": item.field_11,
                        "tag": item.tag,
                        "supported_types": item.supported_types.iter().map(|t| &t.mime_type).collect::<Vec<_>>(),
                    })
                })
                .collect();

            let recommended = m.recommended.as_ref().map(|r| {
                serde_json::json!({
                    "category": r.category,
                    "model_names": r.list.as_ref().map(|l| &l.model_names),
                })
            });

            serde_json::json!({
                "items": items,
                "recommended": recommended,
                "default_model": m.default_model.as_ref().and_then(|d| d.model.as_ref().map(|m| m.id)),
            })
        });

    // 订阅计划 (from context.plan)
    let plan = msg
        .context
        .as_ref()
        .and_then(|ctx| ctx.plan.as_ref())
        .map(|p| {
            serde_json::json!({
                "tier_id": p.tier_id,
                "tier_name": p.tier_name,
                "display_name": p.display_name,
                "upgrade_url": p.upgrade_url,
                "upgrade_message": p.upgrade_message,
            })
        });

    // 用户上下文
    let context = msg.context.as_ref().map(|ctx| {
        serde_json::json!({
            "status": ctx.status,
            "plan_name": ctx.plan_name,
            "email": ctx.email,
            "models": models,
            "plan": plan,
        })
    });

    // 顶层订阅信息
    let subscription = msg.subscription.as_ref().map(|s| {
        serde_json::json!({
            "tier_id": s.tier_id,
            "tier_name": s.tier_name,
            "display_name": s.display_name,
            "upgrade_url": s.upgrade_url,
            "upgrade_message": s.upgrade_message,
        })
    });

    serde_json::json!({
        "field_5_base64": b64(&msg.field_5),
        "auth": auth,
        "field_7_base64": b64(&msg.field_7),
        "field_9_base64": b64(&msg.field_9),
        "field_10_base64": b64(&msg.field_10),
        "field_11_base64": b64(&msg.field_11),
        "field_15_base64": b64(&msg.field_15),
        "field_16_base64": b64(&msg.field_16),
        "field_17_base64": b64(&msg.field_17),
        "f18_base64": b64(&msg.f18),
        "context": context,
        "subscription": subscription,
    })
}
