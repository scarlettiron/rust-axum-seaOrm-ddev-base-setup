use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum SyncEvent {
    Table,
    ConnectionRunId,
}

#[derive(DeriveIden)]
enum ConnectionRun {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum SyncEventIndexes {
    SyncEventConnectionRunIdIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SyncEvent::Table)
                    .add_column(
                        ColumnDef::new(SyncEvent::ConnectionRunId)
                            .big_integer()
                            .null(),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_sync_event_connection_run_id")
                            .from_tbl(SyncEvent::Table)
                            .from_col(SyncEvent::ConnectionRunId)
                            .to_tbl(ConnectionRun::Table)
                            .to_col(ConnectionRun::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SyncEventIndexes::SyncEventConnectionRunIdIdx.to_string())
                    .table(SyncEvent::Table)
                    .col(SyncEvent::ConnectionRunId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

}
