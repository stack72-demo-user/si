use axum::Json;
use serde::{Deserialize, Serialize};

use dal::{
    ComponentId, Func, FuncBackendKind, FuncId, HistoryActor, ReadTenancy, SchemaVariantId,
    StandardModel, Visibility, WriteTenancy, WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::dev::DevError;

use super::DevResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveBuiltinFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
    pub components: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveBuiltinFuncResponse {
    pub success: bool,
}

pub async fn save_builtin_func(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveBuiltinFuncRequest>,
) -> DevResult<Json<SaveBuiltinFuncResponse>> {
    let txns = txns.start().await?;

    let universal_req_ctx = dal::context::AccessBuilder::new(
        ReadTenancy::new_universal(),
        WriteTenancy::new_universal(),
        HistoryActor::SystemInit,
    );
    let ctx = builder.build(universal_req_ctx.build(Visibility::new_head(false)), &txns);

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(DevError::FuncNotFound)?;

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_backend_kind(&ctx, request.kind).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    dal::builtins::func::persist(&func).await?;

    let acct_ctx = builder.build(request_ctx.build(request.visibility), &txns);
    WsEvent::change_set_written(&acct_ctx)
        .publish(&acct_ctx)
        .await?;

    txns.commit().await?;

    Ok(Json(SaveBuiltinFuncResponse { success: true }))
}
