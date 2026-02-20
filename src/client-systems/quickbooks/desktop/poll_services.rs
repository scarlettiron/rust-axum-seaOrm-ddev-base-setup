//! QuickBooks Desktop Web Connector poll service.
//!
//! Two-phase protocol:
//!
//! **Request phase** (`handle_request`):
//!   1. Validate credentials → 403 if invalid
//!   2. Ensure an `erp_connection_sync_state` row exists for the connection
//!   3. Look up the single recurring List/Inventory sync event for this connection
//!      - If none exists → create ConnectionRun + SyncEvent (status = InProgress)
//!      - If Pending or Error → create a fresh ConnectionRun for *this* poll cycle,
//!        update the event to InProgress, increment attempts
//!   4. Build an `ItemInventoryQueryRq` using the cursor stored in `sync_state`
//!      (iterator="Continue" + iteratorID) or a fresh Start if no cursor
//!   5. Return the QBXML string plus UUIDs the caller must echo back in the response phase
//!
//! **Response phase** (`handle_response`):
//!   1. Validate credentials
//!   2. If QBD returned an error → mark event Error + run Error, return
//!   3. Parse the XML response (ItemInventoryQueryRs)
//!   4. Upsert each ItemInventoryRet into `inventory_record` / `inventory_record_event`
//!      - Match on `system_id_key=Qbd` + `system_id={ListID}` + `connection_id`
//!      - Create record+event if new; update latest event if existing
//!   5. Update the cursor in `sync_state` (None if pagination complete)
//!   6. Mark sync event:
//!      - List events → **Pending** (never Completed; will be re-run)
//!      - Other methods → **Success** (or Error on failure)
//!   7. Update ConnectionRun to Error on failure (stays Success otherwise)

use std::collections::HashMap;

use entity::sea_orm_active_enums::{
    ConnectionRunStatus, ConnectionRunType, ErpProvider, ErpProviderType,
    SyncEventCategory, SyncEventDirection, SyncEventMethod, SyncEventStatus, SystemIdKey,
};
use entity::{
    connection_identity, connection_run, erp_connection_credentials, erp_connection_sync_state,
    inventory_record, inventory_record_event, sync_event,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::connection_run::services::{
    ConnectionRunService, CreateConnectionRun, UpdateConnectionRun,
};
use crate::erp_connection_sync_state::services::{
    CreateErpConnectionSyncState, ErpConnectionSyncStateService,
};
use crate::inventory_records::events_services::{
    CreateInventoryRecordEvent, InventoryRecordEventService, UpdateInventoryRecordEvent,
};
use crate::inventory_records::services::{CreateInventoryRecord, InventoryRecordService};
use crate::sync_event::services::{CreateSyncEvent, SyncEventService, UpdateSyncEvent};

/// Items returned per QBXML page.
const PAGE_SIZE: u32 = 50;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum QbdPollError {
    Unauthorized,
    Db(DbErr),
    XmlParse(String),
}

impl From<DbErr> for QbdPollError {
    fn from(e: DbErr) -> Self {
        QbdPollError::Db(e)
    }
}

// ── Public I/O types ──────────────────────────────────────────────────────────

/// Output of `handle_request` (maps to sendRequestXML).
pub struct PollRequestOutput {
    /// Whether there is an inventory sync to perform.
    pub has_work: bool,
    /// QBXML to send to QuickBooks Desktop (None when has_work is false).
    pub xml: Option<String>,
}

/// Input for `handle_response` (maps to receiveResponseXML).
pub struct PollResponseInput {
    /// Full QBXML response string from QuickBooks Desktop.
    pub qbd_response_xml: Option<String>,
    /// Human-readable error returned by QBD (when QBD returned an error instead of XML).
    pub qbd_error: Option<String>,
}

/// Output of `handle_response`.
pub struct PollResponseOutput {
    /// True when there are more pages to fetch (cursor not exhausted).
    /// Maps to QBWC's receiveResponseXML return value: 100 = keep going, 0 = done.
    pub has_more: bool,
}

// ── Internal parsed types ─────────────────────────────────────────────────────

struct ParsedInventoryResponse {
    iterator_id: Option<String>,
    /// Items remaining after this page; 0 means pagination is complete.
    remaining_count: i64,
    status_code: String,
    status_message: String,
    items: Vec<QbdInventoryItem>,
}

struct QbdInventoryItem {
    /// QBD ListID — used as the `system_id`.
    list_id: String,
    name: Option<String>,
    full_name: Option<String>,
    /// Sales price converted to integer cents.
    sales_price_cents: Option<i32>,
    qty_on_hand: Option<i32>,
    sales_desc: Option<String>,
    /// All parsed fields as a JSON blob stored in `original_record_body`.
    raw: Value,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct QbdPollService {
    db: DatabaseConnection,
}

impl QbdPollService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ── Request phase ─────────────────────────────────────────────────────────

    /// Find (or create) the recurring List/Inventory sync event and return the
    /// QBXML query to execute against QuickBooks Desktop.
    pub async fn handle_request(
        &self,
        username: &str,
        password: &str,
    ) -> Result<PollRequestOutput, QbdPollError> {
        let (conn, _creds) = self.validate_credentials(username, password).await?;
        let sync_state = self.ensure_sync_state(conn.id).await?;

        let run_svc = ConnectionRunService::new(self.db.clone());
        let sync_event_svc = SyncEventService::new(self.db.clone());

        // Find the ONE recurring List/Inventory event for this connection that
        // is ready to be processed (Pending or Error).
        let maybe_event = sync_event::Entity::find()
            .filter(
                Condition::any()
                    .add(sync_event::Column::Status.eq(SyncEventStatus::Pending))
                    .add(sync_event::Column::Status.eq(SyncEventStatus::Error)),
            )
            .filter(sync_event::Column::ConnectionSyncStateId.eq(sync_state.id))
            .filter(sync_event::Column::SyncEventMethod.eq(SyncEventMethod::List))
            .filter(sync_event::Column::SyncEventCategory.eq(SyncEventCategory::Inventory))
            .one(&self.db)
            .await?;

        // Build the cursor XML now (before we mutate the event).
        let cursor = sync_state.sync_cursor.clone();
        let xml = build_item_inventory_query_xml(cursor.as_ref());

        match maybe_event {
            None => {
                // First ever poll — create a fresh event and run.
                let run = run_svc
                    .create(
                        CreateConnectionRun {
                            connection_id: conn.id,
                            status: Some(ConnectionRunStatus::Success),
                            run_type: Some(ConnectionRunType::Poll),
                            error_message: None,
                        },
                        None,
                    )
                    .await?;

                let _event = sync_event_svc
                    .create(
                        CreateSyncEvent {
                            original_record_body: None,
                            details: None,
                            event_direction: SyncEventDirection::PullFromExternal,
                            inventory_record_event_id: None,
                            sync_event_method: SyncEventMethod::List,
                            sync_event_category: SyncEventCategory::Inventory,
                            attempts: Some(1),
                            status: Some(SyncEventStatus::InProgress),
                            last_error: None,
                            last_errored_date: None,
                            connection_sync_state_id: Some(sync_state.id),
                            connection_run_id: Some(run.id),
                        },
                        None,
                    )
                    .await?;
            }

            Some(event) => {
                // Existing event — create a NEW ConnectionRun per poll per event.
                let run = run_svc
                    .create(
                        CreateConnectionRun {
                            connection_id: conn.id,
                            status: Some(ConnectionRunStatus::Success),
                            run_type: Some(ConnectionRunType::Poll),
                            error_message: None,
                        },
                        None,
                    )
                    .await?;

                // Mark InProgress and link to the new run.
                let _ = sync_event_svc
                    .update_by_uuid(
                        event.uuid,
                        UpdateSyncEvent {
                            status: Some(SyncEventStatus::InProgress),
                            attempts: Some(event.attempts + 1),
                            connection_run_id: Some(run.id),
                            original_record_body: None,
                            details: None,
                            event_direction: None,
                            inventory_record_event_id: None,
                            sync_event_method: None,
                            sync_event_category: None,
                            last_error: None,
                            last_errored_date: None,
                            connection_sync_state_id: None,
                        },
                        None,
                    )
                    .await;
            }
        }

        Ok(PollRequestOutput {
            has_work: true,
            xml: Some(xml),
        })
    }

    // ── Response phase ────────────────────────────────────────────────────────

    /// Process the XML response returned by QuickBooks Desktop (receiveResponseXML).
    ///
    /// Finds the current InProgress sync event for this connection — no UUID echoing
    /// required, the server tracks all state. Returns `has_more` so the adapter can
    /// signal QBWC to call sendRequestXML again (100) or stop (0).
    pub async fn handle_response(
        &self,
        username: &str,
        password: &str,
        input: PollResponseInput,
    ) -> Result<PollResponseOutput, QbdPollError> {
        let (conn, _creds) = self.validate_credentials(username, password).await?;
        let sync_state = self.ensure_sync_state(conn.id).await?;

        let sync_event_svc = SyncEventService::new(self.db.clone());
        let run_svc = ConnectionRunService::new(self.db.clone());

        // Find the InProgress List/Inventory event for this connection.
        // There should be at most one at a time since handle_request marks it
        // InProgress before returning the QBXML to the adapter.
        let event = sync_event::Entity::find()
            .filter(sync_event::Column::ConnectionSyncStateId.eq(sync_state.id))
            .filter(sync_event::Column::Status.eq(SyncEventStatus::InProgress))
            .filter(sync_event::Column::SyncEventMethod.eq(SyncEventMethod::List))
            .filter(sync_event::Column::SyncEventCategory.eq(SyncEventCategory::Inventory))
            .one(&self.db)
            .await?;

        // Load the ConnectionRun created when we sent the request.
        let run = if let Some(ref ev) = event {
            if let Some(run_id) = ev.connection_run_id {
                connection_run::Entity::find_by_id(run_id)
                    .one(&self.db)
                    .await?
            } else {
                None
            }
        } else {
            None
        };

        // ── QBD returned an error ─────────────────────────────────────────────
        if let Some(ref err_msg) = input.qbd_error {
            let err_body = json!({ "message": err_msg });

            if let Some(ref ev) = event {
                let _ = sync_event_svc
                    .update_by_uuid(
                        ev.uuid,
                        UpdateSyncEvent {
                            status: Some(SyncEventStatus::Error),
                            last_error: Some(err_body.clone()),
                            last_errored_date: Some(chrono::Utc::now()),
                            attempts: None,
                            original_record_body: None,
                            details: None,
                            event_direction: None,
                            inventory_record_event_id: None,
                            sync_event_method: None,
                            sync_event_category: None,
                            connection_sync_state_id: None,
                            connection_run_id: None,
                        },
                        None,
                    )
                    .await;
            }

            if let Some(ref r) = run {
                let _ = run_svc
                    .update_by_uuid(
                        r.uuid,
                        UpdateConnectionRun {
                            status: Some(ConnectionRunStatus::Error),
                            error_message: Some(err_msg.clone()),
                        },
                        None,
                    )
                    .await;
            }

            return Ok(PollResponseOutput { has_more: false });
        }

        // ── Parse XML ─────────────────────────────────────────────────────────
        let xml_str = match input.qbd_response_xml.as_deref() {
            Some(x) => x,
            None => return Ok(PollResponseOutput { has_more: false }),
        };

        let parsed = match parse_inventory_response(xml_str) {
            Ok(p) => p,
            Err(e) => {
                let msg = format!("XML parse error: {e}");
                self.mark_event_and_run_error(&event, &run, &msg, &sync_event_svc, &run_svc)
                    .await;
                return Err(QbdPollError::XmlParse(msg));
            }
        };

        // QBD can return statusCode != "0" as a soft error inside the XML.
        if parsed.status_code != "0" {
            let msg = format!(
                "QBD status {}: {}",
                parsed.status_code, parsed.status_message
            );
            self.mark_event_and_run_error(&event, &run, &msg, &sync_event_svc, &run_svc)
                .await;
            return Err(QbdPollError::XmlParse(msg));
        }

        // ── Upsert inventory items ────────────────────────────────────────────
        let mut errors: Vec<String> = Vec::new();
        for item in &parsed.items {
            if let Err(e) = self.upsert_inventory_item(&conn, item).await {
                errors.push(format!("ListID={}: {:?}", item.list_id, e));
            }
        }

        // ── Update cursor in sync_state ───────────────────────────────────────
        // remaining_count > 0 → store iteratorID so next sendRequestXML uses Continue.
        // remaining_count = 0 → clear cursor so next poll starts fresh with iterator="Start".
        let new_cursor = if parsed.remaining_count > 0 {
            Some(json!({
                "iterator_id": parsed.iterator_id,
                "remaining_count": parsed.remaining_count,
            }))
        } else {
            None
        };

        {
            let sync_state_svc = ErpConnectionSyncStateService::new(self.db.clone());
            if let Ok(Some(ss)) = sync_state_svc.get_by_id(sync_state.id, None).await {
                // Direct ActiveModel update so we can set cursor to NULL.
                let mut active: erp_connection_sync_state::ActiveModel = ss.into();
                active.sync_cursor = Set(new_cursor);
                active.updated_at = Set(chrono::Utc::now().into());
                let _ = active.update(&self.db).await;
            }
        }

        // ── Mark sync event and run ───────────────────────────────────────────
        let has_errors = !errors.is_empty();

        if let Some(ref ev) = event {
            let is_list = ev.sync_event_method == SyncEventMethod::List;

            let (new_status, last_error) = if has_errors {
                let err_body = json!({ "errors": errors });
                // List events go back to Pending even with partial errors so they
                // are retried; error info is preserved.
                let status = if is_list {
                    SyncEventStatus::Pending
                } else {
                    SyncEventStatus::Error
                };
                (status, Some(err_body))
            } else {
                // List events: Pending so the next sendRequestXML picks them up.
                // Other events: Success.
                let status = if is_list {
                    SyncEventStatus::Pending
                } else {
                    SyncEventStatus::Success
                };
                (status, None)
            };

            let _ = sync_event_svc
                .update_by_uuid(
                    ev.uuid,
                    UpdateSyncEvent {
                        status: Some(new_status),
                        last_error,
                        last_errored_date: if has_errors {
                            Some(chrono::Utc::now())
                        } else {
                            None
                        },
                        attempts: None,
                        original_record_body: None,
                        details: None,
                        event_direction: None,
                        inventory_record_event_id: None,
                        sync_event_method: None,
                        sync_event_category: None,
                        connection_sync_state_id: None,
                        connection_run_id: None,
                    },
                    None,
                )
                .await;
        }

        if has_errors {
            if let Some(ref r) = run {
                let _ = run_svc
                    .update_by_uuid(
                        r.uuid,
                        UpdateConnectionRun {
                            status: Some(ConnectionRunStatus::Error),
                            error_message: Some(errors.join("; ")),
                        },
                        None,
                    )
                    .await;
            }
        }

        // has_more = true  → adapter returns 100 to QBWC (call sendRequestXML again immediately)
        // has_more = false → adapter returns 0 to QBWC (stop until next scheduled poll)
        Ok(PollResponseOutput {
            has_more: parsed.remaining_count > 0,
        })
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    async fn validate_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<
        (
            connection_identity::Model,
            erp_connection_credentials::Model,
        ),
        QbdPollError,
    > {
        let creds = erp_connection_credentials::Entity::find()
            .filter(erp_connection_credentials::Column::ProviderUserId.eq(username))
            .one(&self.db)
            .await?
            .ok_or(QbdPollError::Unauthorized)?;

        if creds.provider_password.as_deref().unwrap_or("") != password {
            return Err(QbdPollError::Unauthorized);
        }

        let conn = connection_identity::Entity::find_by_id(creds.connection_id)
            .one(&self.db)
            .await?
            .ok_or(QbdPollError::Unauthorized)?;

        if conn.erp_provider != ErpProvider::Quickbooks
            || conn.erp_type != ErpProviderType::Desktop
        {
            return Err(QbdPollError::Unauthorized);
        }

        Ok((conn, creds))
    }

    async fn ensure_sync_state(
        &self,
        connection_id: i64,
    ) -> Result<erp_connection_sync_state::Model, QbdPollError> {
        let svc = ErpConnectionSyncStateService::new(self.db.clone());
        match svc.get_by_connection_id(connection_id, None).await? {
            Some(s) => Ok(s),
            None => Ok(svc
                .create(
                    CreateErpConnectionSyncState {
                        connection_id,
                        sync_cursor: None,
                        sync_lock_owner: None,
                        sync_lock_until: None,
                        rate_limit_remaining: None,
                        rate_limit: None,
                        rate_limit_reset_at: None,
                        rate_limit_backoff_until: None,
                        rate_limit_window_seconds: None,
                    },
                    None,
                )
                .await?),
        }
    }

    /// Create or update a single inventory item from a QBD response.
    ///
    /// - Matches on `system_id_key=Qbd` + `system_id={ListID}` + `originating_connection_id`
    /// - Creates `inventory_record` + `inventory_record_event` if new
    /// - Updates the most recent `inventory_record_event` if the record already exists
    async fn upsert_inventory_item(
        &self,
        conn: &connection_identity::Model,
        item: &QbdInventoryItem,
    ) -> Result<(), QbdPollError> {
        let inv_svc = InventoryRecordService::new(self.db.clone());
        let evt_svc = InventoryRecordEventService::new(self.db.clone());

        // Look up the canonical inventory record by QBD ListID.
        let record = inventory_record::Entity::find()
            .filter(inventory_record::Column::SystemIdKey.eq(SystemIdKey::Qbd))
            .filter(inventory_record::Column::SystemId.eq(&item.list_id))
            .filter(inventory_record::Column::OriginatingConnectionId.eq(conn.id))
            .one(&self.db)
            .await?;

        let record = match record {
            Some(r) => {
                // Refresh the raw body on the parent record.
                let _ = inv_svc
                    .update_by_id(
                        r.id,
                        crate::inventory_records::services::UpdateInventoryRecord {
                            original_record_body: Some(item.raw.clone()),
                            system_id_key: None,
                            system_id: None,
                        },
                        None,
                    )
                    .await;
                r
            }
            None => {
                inv_svc
                    .create(
                        CreateInventoryRecord {
                            tenant_id: conn.tenant_id,
                            originating_connection_id: conn.id,
                            original_record_body: Some(item.raw.clone()),
                            system_id_key: SystemIdKey::Qbd,
                            system_id: item.list_id.clone(),
                        },
                        None,
                    )
                    .await?
            }
        };

        // Find the most recent event for this record + connection.
        let existing_event = inventory_record_event::Entity::find()
            .filter(inventory_record_event::Column::InventoryRecordId.eq(record.id))
            .filter(inventory_record_event::Column::ConnectionId.eq(conn.id))
            .order_by_desc(inventory_record_event::Column::CreatedAt)
            .one(&self.db)
            .await?;

        match existing_event {
            Some(ev) => {
                let _ = evt_svc
                    .update_by_id(
                        ev.id,
                        UpdateInventoryRecordEvent {
                            original_record_body: Some(item.raw.clone()),
                            price: item.sales_price_cents,
                            currency: None,
                            name: item.name.clone(),
                            description: item.sales_desc.clone(),
                            attributes: None,
                            qty: item.qty_on_hand,
                            external_code: item.full_name.clone(),
                        },
                        None,
                    )
                    .await;
            }
            None => {
                evt_svc
                    .create(
                        CreateInventoryRecordEvent {
                            inventory_record_id: record.id,
                            connection_id: conn.id,
                            original_record_body: Some(item.raw.clone()),
                            price: item.sales_price_cents,
                            currency: None,
                            name: item.name.clone(),
                            description: item.sales_desc.clone(),
                            attributes: None,
                            qty: item.qty_on_hand,
                            external_code: item.full_name.clone(),
                        },
                        None,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Best-effort: mark a sync event and connection run as Error.
    async fn mark_event_and_run_error(
        &self,
        event: &Option<sync_event::Model>,
        run: &Option<connection_run::Model>,
        message: &str,
        sync_event_svc: &SyncEventService,
        run_svc: &ConnectionRunService,
    ) {
        let err_body = json!({ "message": message });

        if let Some(ev) = event {
            let _ = sync_event_svc
                .update_by_uuid(
                    ev.uuid,
                    UpdateSyncEvent {
                        status: Some(SyncEventStatus::Error),
                        last_error: Some(err_body),
                        last_errored_date: Some(chrono::Utc::now()),
                        attempts: None,
                        original_record_body: None,
                        details: None,
                        event_direction: None,
                        inventory_record_event_id: None,
                        sync_event_method: None,
                        sync_event_category: None,
                        connection_sync_state_id: None,
                        connection_run_id: None,
                    },
                    None,
                )
                .await;
        }

        if let Some(r) = run {
            let _ = run_svc
                .update_by_uuid(
                    r.uuid,
                    UpdateConnectionRun {
                        status: Some(ConnectionRunStatus::Error),
                        error_message: Some(message.to_string()),
                    },
                    None,
                )
                .await;
        }
    }
}

// ── QBXML builders ────────────────────────────────────────────────────────────

/// Build an `ItemInventoryQueryRq`.
///
/// Uses `iterator="Continue" iteratorID="..."` when a cursor is present,
/// otherwise `iterator="Start"`.
fn build_item_inventory_query_xml(cursor: Option<&Value>) -> String {
    let iterator_id = cursor
        .and_then(|c| c.get("iterator_id"))
        .and_then(|v| v.as_str());

    match iterator_id {
        None => format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<?qbxml version="13.0"?>
<QBXML>
  <QBXMLMsgsRq onError="stopOnError">
    <ItemInventoryQueryRq requestID="1" iterator="Start" maxReturned="{ps}">
    </ItemInventoryQueryRq>
  </QBXMLMsgsRq>
</QBXML>"#,
            ps = PAGE_SIZE
        ),
        Some(id) => format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<?qbxml version="13.0"?>
<QBXML>
  <QBXMLMsgsRq onError="stopOnError">
    <ItemInventoryQueryRq requestID="1" iterator="Continue" iteratorID="{id}" maxReturned="{ps}">
    </ItemInventoryQueryRq>
  </QBXMLMsgsRq>
</QBXML>"#,
            id = id,
            ps = PAGE_SIZE
        ),
    }
}

// ── XML parser ────────────────────────────────────────────────────────────────

/// Parse a QBD `ItemInventoryQueryRs` QBXML response.
fn parse_inventory_response(xml: &str) -> Result<ParsedInventoryResponse, String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut iterator_id: Option<String> = None;
    let mut remaining_count: i64 = 0;
    let mut status_code = "0".to_string();
    let mut status_message = String::new();
    let mut items: Vec<QbdInventoryItem> = Vec::new();

    let mut in_item = false;
    let mut current_tag: Option<String> = None;
    let mut current_data: HashMap<String, String> = HashMap::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name =
                    String::from_utf8_lossy(e.name().as_ref()).to_string();

                match name.as_str() {
                    "ItemInventoryQueryRs" => {
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val =
                                String::from_utf8_lossy(attr.value.as_ref()).to_string();
                            match key.as_str() {
                                "iteratorID" => iterator_id = Some(val),
                                "iteratorRemainingCount" => {
                                    remaining_count = val.parse().unwrap_or(0);
                                }
                                "statusCode" => status_code = val,
                                "statusMessage" => status_message = val,
                                _ => {}
                            }
                        }
                    }
                    "ItemInventoryRet" => {
                        in_item = true;
                        current_data.clear();
                        current_tag = None;
                    }
                    _ if in_item => {
                        current_tag = Some(name);
                    }
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let name =
                    String::from_utf8_lossy(e.name().as_ref()).to_string();

                if name == "ItemInventoryRet" {
                    in_item = false;
                    current_tag = None;

                    if let Some(list_id) = current_data.get("ListID").cloned() {
                        let price_cents = current_data
                            .get("SalesPrice")
                            .and_then(|p| p.parse::<f64>().ok())
                            .map(|p| (p * 100.0).round() as i32);

                        let qty = current_data
                            .get("QuantityOnHand")
                            .and_then(|q| q.parse::<i32>().ok());

                        let raw: Value = current_data
                            .iter()
                            .fold(serde_json::Map::new(), |mut m, (k, v)| {
                                m.insert(k.clone(), Value::String(v.clone()));
                                m
                            })
                            .into();

                        items.push(QbdInventoryItem {
                            list_id,
                            name: current_data.get("Name").cloned(),
                            full_name: current_data.get("FullName").cloned(),
                            sales_price_cents: price_cents,
                            qty_on_hand: qty,
                            sales_desc: current_data.get("SalesDesc").cloned(),
                            raw,
                        });
                    }
                    current_data.clear();
                } else if in_item {
                    current_tag = None;
                }
            }

            Ok(Event::Text(ref e)) if in_item => {
                if let (Some(tag), Ok(text)) = (&current_tag, e.unescape()) {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        current_data.insert(tag.clone(), text);
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("{e}")),
            _ => {}
        }
    }

    Ok(ParsedInventoryResponse {
        iterator_id,
        remaining_count,
        status_code,
        status_message,
        items,
    })
}
